use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

use axum::body::Bytes;
use tokio::sync::OnceCell;

use crate::error::AppError;

/// A minimal in-memory TTL cache for serialized JSON responses.
///
/// Bodies share their allocation across cache hits and concurrent responses.
/// Writes prune expired entries and evict the earliest expiry at capacity.
pub struct Cache {
    inner: RwLock<HashMap<String, Entry>>,
}

struct Entry {
    body: Bytes,
    expires_at: Instant,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
        }
    }

    /// Returns the cached body if present and not expired, evicting the
    /// entry when it has expired.
    pub fn get(&self, key: &str) -> Option<Bytes> {
        {
            let map = self.inner.read().ok()?;
            let entry = map.get(key)?;
            if Instant::now() < entry.expires_at {
                return Some(entry.body.clone());
            }
        }
        // Expired: drop the read lock, then remove under the write lock so a
        // concurrent writer cannot be starved by a long read hold.
        if let Ok(mut map) = self.inner.write() {
            if let Some(entry) = map.get(key) {
                if Instant::now() >= entry.expires_at {
                    map.remove(key);
                }
            }
        }
        None
    }

    /// Stores a body with the given TTL.
    pub fn set(&self, key: &str, body: Bytes, ttl: Duration) {
        if let Ok(mut map) = self.inner.write() {
            let now = Instant::now();
            map.retain(|_, entry| now < entry.expires_at);
            if ttl.is_zero() {
                map.remove(key);
                return;
            }
            if map.len() >= 1024 && !map.contains_key(key) {
                if let Some(oldest) = map.iter().min_by_key(|(_, entry)| entry.expires_at).map(|(key, _)| key.clone()) {
                    map.remove(&oldest);
                }
            }
            map.insert(
                key.to_string(),
                Entry {
                    body,
                    expires_at: now + ttl,
                },
            );
        }
    }

    /// Removes all entries.
    pub fn clear(&self) {
        if let Ok(mut map) = self.inner.write() {
            map.clear();
        }
    }

    /// Number of unexpired cached entries.
    pub fn len(&self) -> usize {
        let now = Instant::now();
        self.inner
            .read()
            .map(|m| m.values().filter(|entry| now < entry.expires_at).count())
            .unwrap_or(0)
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}

/// A shared in-flight outcome cell: the leader's success or failure, cloned by
/// followers once it resolves.
type CoalescedCell = Arc<OnceCell<Result<Bytes, AppError>>>;

/// In-process single-flight for cache misses. Concurrent requests for the same
/// key share one in-flight upstream call and clone its outcome, so a burst on a
/// cold cache produces one game-server request per cache window instead of one
/// per requester.
#[derive(Default)]
pub struct Coalescer {
    inflight: Mutex<HashMap<String, CoalescedCell>>,
}

/// If all callers disconnect while fetching, release the abandoned slot.
struct FlightGuard<'a> {
    owner: &'a Coalescer,
    key: &'a str,
    cell: Option<CoalescedCell>,
}

impl Drop for FlightGuard<'_> {
    fn drop(&mut self) {
        let cell = self.cell.take().unwrap();
        if let Ok(mut inflight) = self.owner.inflight.lock() {
            if inflight.get(self.key).is_some_and(|slot| Arc::ptr_eq(slot, &cell)) && Arc::strong_count(&cell) == 2 {
                inflight.remove(self.key);
            }
            // Drop our reference while holding the lock so simultaneous
            // cancellations cannot both mistake the other for a live waiter.
            drop(cell);
        }
    }
}

impl Coalescer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Runs `fetch` once per in-flight window for `key`; concurrent callers
    /// await the leader's outcome, success or failure. The slot is removed when
    /// the leader finishes so a later request fetches afresh rather than reusing
    /// a stale in-flight result.
    pub async fn run<F, Fut>(&self, key: &str, fetch: F) -> Result<Bytes, AppError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<Bytes, AppError>>,
    {
        let cell = {
            let mut inflight = self.inflight.lock().unwrap();
            inflight.entry(key.to_string()).or_insert_with(|| Arc::new(OnceCell::new())).clone()
        };
        let guard = FlightGuard {
            owner: self,
            key,
            cell: Some(cell),
        };
        let cell = guard.cell.as_ref().unwrap();
        let outcome = cell.get_or_init(fetch).await.clone();
        if let Ok(mut inflight) = self.inflight.lock() {
            if let Some(slot) = inflight.get(key) {
                if Arc::ptr_eq(slot, cell) {
                    inflight.remove(key);
                }
            }
        }
        outcome
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn cache_shares_storage_and_expires_entries() {
        let cache = Cache::new();
        let body = Bytes::from(vec![42; 1024 * 1024]);
        cache.set("a", body.clone(), Duration::from_secs(60));
        assert_eq!(cache.get("a").unwrap().as_ptr(), body.as_ptr());
        cache.inner.write().unwrap().get_mut("a").unwrap().expires_at = Instant::now();
        assert_eq!(cache.len(), 0);
        cache.set("b", Bytes::new(), Duration::from_secs(60));
        assert!(!cache.inner.read().unwrap().contains_key("a"));
        cache.set("b", Bytes::new(), Duration::ZERO);
        assert!(cache.get("b").is_none());
        cache.clear();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn cache_is_bounded_and_replacing_an_entry_does_not_evict_another() {
        let cache = Cache::new();
        for id in 0..1025 {
            cache.set(&id.to_string(), Bytes::new(), Duration::from_secs(60));
        }
        assert_eq!(cache.len(), 1024);
        assert!(cache.get("0").is_none());
        cache.set("1024", Bytes::from_static(b"new"), Duration::from_secs(60));
        assert_eq!(cache.len(), 1024);
    }

    #[tokio::test]
    async fn concurrent_callers_share_success_and_failure() {
        for fail in [false, true] {
            let coalescer = Coalescer::new();
            let calls = AtomicUsize::new(0);
            let fetch = || async {
                calls.fetch_add(1, Ordering::SeqCst);
                tokio::task::yield_now().await;
                if fail {
                    Err(AppError::Upstream(503))
                } else {
                    Ok(Bytes::from_static(b"ok"))
                }
            };
            let (a, b, c) = tokio::join!(
                coalescer.run("key", fetch),
                coalescer.run("key", fetch),
                coalescer.run("key", fetch)
            );
            assert_eq!(calls.load(Ordering::SeqCst), 1);
            assert_eq!(a.is_err(), fail);
            assert_eq!(b.is_err(), fail);
            assert_eq!(c.is_err(), fail);
            assert!(coalescer.inflight.lock().unwrap().is_empty());
            let _ = coalescer.run("key", fetch).await;
            assert_eq!(calls.load(Ordering::SeqCst), 2);
        }
    }

    #[tokio::test]
    async fn cancelled_fetch_does_not_leak_a_slot() {
        let coalescer = Arc::new(Coalescer::new());
        let (started, ready) = tokio::sync::oneshot::channel();
        let task = tokio::spawn({
            let coalescer = coalescer.clone();
            async move {
                coalescer
                    .run("key", || async {
                        started.send(()).unwrap();
                        std::future::pending().await
                    })
                    .await
            }
        });
        ready.await.unwrap();
        task.abort();
        assert!(task.await.unwrap_err().is_cancelled());
        assert!(coalescer.inflight.lock().unwrap().is_empty());
        assert_eq!(
            coalescer.run("key", || async { Ok(Bytes::from_static(b"retry")) }).await.unwrap(),
            "retry"
        );
    }
}
