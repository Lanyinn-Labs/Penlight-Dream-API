//! Garupa official-game API client: request signing and AES decryption.
//! Ported from GarupaSpeedTracker's `backend/src/api/garupa.ts`, keeping only
//! the non-CN path since this project targets the JP server.

use std::io::Read;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use axum::body::Bytes;
use bzip2::read::BzDecoder;
use reqwest::header::{HeaderMap, HeaderValue};
use tracing::warn;

use crate::config::{Config, ServerConfig};
use crate::crypto::decrypt_aes_128_cbc;
use crate::error::{AppError, AppResult};

/// Builds the request headers required by the Garupa API.
fn build_headers(cfg: &ServerConfig, client_version: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();

    fn hv(s: &str) -> HeaderValue {
        HeaderValue::from_str(s).unwrap_or_else(|_| HeaderValue::from_static(""))
    }

    headers.insert("User-Agent", hv(&cfg.user_agent));
    headers.insert("X-Unity-Version", hv(&cfg.unity_version));
    headers.insert("X-ClientPlatform", hv(&cfg.client_platform));
    headers.insert("X-ClientVersion", hv(client_version));
    headers.insert("X-Signature", hv(&cfg.uuid));
    headers.insert("Accept-Encoding", hv("deflate, gzip"));
    headers.insert("Content-Type", hv("application/octet-stream"));
    headers.insert("Accept", hv("application/octet-stream"));

    headers
}

/// Caches the auto-detected client version, refreshed from the App Store lookup.
struct VersionCache {
    inner: Mutex<Option<(String, Instant)>>,
    ttl: Duration,
    http: reqwest::Client,
}

impl VersionCache {
    fn new(ttl: Duration, http: reqwest::Client) -> Self {
        Self {
            inner: Mutex::new(None),
            ttl,
            http,
        }
    }

    /// Returns the current client version, refreshing from the App Store lookup
    /// URL when the cache is stale. Falls back to the configured static version
    /// on any lookup failure.
    async fn get(&self, cfg: &ServerConfig) -> String {
        if let Some((version, at)) = self.inner.lock().unwrap().clone() {
            if at.elapsed() < self.ttl {
                return version;
            }
        }

        match self.fetch_from_store(cfg).await {
            Ok(version) => {
                self.inner.lock().unwrap().replace((version.clone(), Instant::now()));
                version
            }
            Err(e) => {
                warn!("client version lookup failed: {e}");
                if let Some((version, _)) = self.inner.lock().unwrap().clone() {
                    version
                } else {
                    cfg.client_version.clone()
                }
            }
        }
    }

    /// Drops the cached version so the next read refetches from the store.
    /// Used when the game API reports the client version is out of date.
    fn drop_cache(&self) {
        self.inner.lock().unwrap().take();
    }

    async fn fetch_from_store(&self, cfg: &ServerConfig) -> Result<String, AppError> {
        if cfg.package_url.is_empty() {
            return Ok(cfg.client_version.clone());
        }
        let resp = self.http.get(&cfg.package_url).timeout(Duration::from_secs(5)).send().await?;
        let json: serde_json::Value = resp.json().await?;
        json["results"][0]["version"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| AppError::Internal("client version not found in App Store response".to_string()))
    }
}

/// The Garupa game API client.
pub struct GarupaClient {
    http: reqwest::Client,
    versions: VersionCache,
}

struct FetchRaw {
    status: u16,
    body: Bytes,
    encoding: Option<String>,
}

impl GarupaClient {
    pub fn new(config: &Config) -> AppResult<Self> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_millis(config.http_timeout_ms))
            .build()
            .map_err(|e| AppError::Internal(format!("failed to build HTTP client: {e}")))?;

        let versions = VersionCache::new(Duration::from_secs(config.version_ttl_secs), http.clone());

        Ok(Self { http, versions })
    }

    /// Returns the current client version, auto-detected with a static fallback.
    pub async fn client_version(&self, cfg: &ServerConfig) -> String {
        self.versions.get(cfg).await
    }

    /// Fetches a URL with the given headers and returns the status and raw body.
    async fn fetch_raw(&self, url: &str, headers: HeaderMap) -> AppResult<FetchRaw> {
        let resp = self.http.get(url).headers(headers).send().await?;
        let status = resp.status().as_u16();
        let encoding = resp
            .headers()
            .get("X-Encoding")
            .and_then(|value| value.to_str().ok())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_ascii_lowercase);
        let body = resp
            .bytes()
            .await
            .map_err(|e| AppError::UpstreamError(format!("failed to read upstream body: {e}")))?;
        Ok(FetchRaw { status, body, encoding })
    }

    /// Decrypts a response and applies Garupa's endpoint-specific compression.
    /// Most endpoints return plaintext protobuf after AES decryption, while the
    /// full suite master endpoint advertises a bzip2 payload in `X-Encoding`.
    fn decode_response_body(&self, cfg: &ServerConfig, raw: &FetchRaw) -> AppResult<Vec<u8>> {
        let decrypted = decrypt_aes_128_cbc(&cfg.encryption_key, &cfg.encryption_iv, &raw.body)?;
        match raw.encoding.as_deref() {
            None => Ok(decrypted),
            Some("bzip2") => decompress_bzip2(&decrypted),
            Some(encoding) => Err(AppError::UpstreamError(format!(
                "unsupported upstream response encoding: {encoding}"
            ))),
        }
    }

    fn expect_success(&self, raw: &FetchRaw) -> AppResult<()> {
        if (200..300).contains(&raw.status) {
            Ok(())
        } else {
            Err(AppError::Upstream(raw.status))
        }
    }

    async fn fetch_with_current_version(&self, cfg: &ServerConfig, url: &str) -> AppResult<FetchRaw> {
        let client_version = self.client_version(cfg).await;
        self.fetch_raw(url, build_headers(cfg, &client_version)).await
    }

    /// Fetches a Garupa endpoint and returns the decrypted protobuf bytes.
    ///
    /// A 426 response means the game updated and the current client version is
    /// stale; the version cache is dropped, refetched from the store, and the
    /// request retried once before the error is surfaced.
    pub async fn fetch(&self, cfg: &ServerConfig, url: &str) -> AppResult<Vec<u8>> {
        let mut raw = self.fetch_with_current_version(cfg, url).await?;
        if raw.status == 426 {
            self.versions.drop_cache();
            raw = self.fetch_with_current_version(cfg, url).await?;
        }
        self.expect_success(&raw)?;
        self.decode_response_body(cfg, &raw)
    }

    /// Checks whether the server is reachable by hitting its `/application` endpoint.
    pub async fn check_health(&self, cfg: &ServerConfig) -> bool {
        let url = format!("{}application", cfg.base);
        let headers = build_headers(cfg, &cfg.client_version);
        match self.http.get(url).headers(headers).timeout(Duration::from_secs(3)).send().await {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }
}

// ============================================================================
// URL builders used by the API handlers
// ============================================================================

macro_rules! endpoint_urls {
    ($($name:ident => $path:literal),+ $(,)?) => {
        $(
            pub fn $name(&self, cfg: &ServerConfig) -> String {
                format!("{}{}", cfg.base, $path)
            }
        )+
    };
}

macro_rules! user_resource_urls {
    ($($name:ident => $path:literal),+ $(,)?) => {
        $(
            pub fn $name(&self, cfg: &ServerConfig) -> String {
                format!("{}user/{}/{}", cfg.base, cfg.uid, $path)
            }
        )+
    };
}

impl GarupaClient {
    endpoint_urls! {
        monthly_ranking_master_url => "monthlyranking",
        event_master_url => "event",
        application_url => "application",
        music_master_url => "music",
        music_difficulty_master_url => "musicdifficulty",
        multi_live_difficulty_master_url => "multilivedifficulty",
        weekly_multi_live_difficulty_master_url => "weeklymultilivedifficulty",
        area_item_master_url => "areaitem",
        area_item_spawn_master_url => "areaitemspawn",
        bonds_master_url => "bonds",
        bonds_effect_master_url => "bondseffect",
        action_set_master_url => "actionset",
        music_shop_master_url => "musicshop",
        degree_master_url => "degree",
        suite_master_url => "suite/master",
        character_master_url => "character",
        band_master_url => "band",
        area_master_url => "area",
        gacha_master_url => "gacha",
        item_master_url => "item",
        skill_master_url => "skill",
        stamp_master_url => "stamp",
        login_bonus_master_url => "loginbonus",
        costume_master_url => "costume",
        shop_url => "shop",
        situation_master_url => "situation",
    }

    user_resource_urls! {
        user_deck_url => "deck",
        user_situation_url => "situation",
        user_title_url => "title",
        user_stamp_url => "stamp",
        user_area_url => "area",
        user_item_url => "item",
        user_present_url => "present",
        user_gacha_url => "gacha",
        user_episode_url => "episode",
        user_mission_url => "mission",
        user_login_bonus_url => "loginbonus",
        user_costume_url => "costume",
        user_character_url => "character",
    }

    pub fn monthly_ranking_url(&self, cfg: &ServerConfig, monthly_id: i64) -> String {
        format!("{}user/{}/monthlyranking/{}/ranking", cfg.base, cfg.uid, monthly_id)
    }

    pub fn user_profile_url(&self, cfg: &ServerConfig) -> String {
        format!("{}user/{}", cfg.base, cfg.uid)
    }

    /// Returns the authenticated user's complete game-data snapshot.
    ///
    /// Unlike the individual `/user/{uid}/...` resources, this suite response
    /// contains the area-item map and character-rank map used by the profile
    /// screen.
    pub fn suite_user_url(&self, cfg: &ServerConfig) -> String {
        format!("{}suite/user/{}", cfg.base, cfg.uid)
    }

    pub fn music_single_url(&self, cfg: &ServerConfig, music_id: i64) -> String {
        format!("{}music/{music_id}", cfg.base)
    }

    pub fn character_single_url(&self, cfg: &ServerConfig, character_id: i64) -> String {
        format!("{}character/{character_id}", cfg.base)
    }

    pub fn event_ranking_url(&self, cfg: &ServerConfig, event_id: i64, event_type: &str, mid: Option<i64>) -> String {
        let segment = event_type_to_url_segment(event_type);
        let mut url = format!("{}user/{}/event/{}/{}/ranking", cfg.base, cfg.uid, event_id, segment);
        if let Some(m) = mid {
            url.push_str(&format!("?mid={m}"));
        }
        url
    }
}

/// Maps a protobuf event type string to its API URL path segment.
fn event_type_to_url_segment(event_type: &str) -> &str {
    match event_type {
        "challenge" => "challenge",
        "live_try" => "livetry",
        "medley" => "medley",
        "mission_live" => "mission",
        "story" => "story",
        "team_live_festival" => "festival",
        "versus" => "versus",
        other => other,
    }
}

fn decompress_bzip2(body: &[u8]) -> AppResult<Vec<u8>> {
    let mut decoder = BzDecoder::new(body);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    Ok(decompressed)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use bzip2::write::BzEncoder;
    use bzip2::Compression;

    use super::decompress_bzip2;

    #[test]
    fn decompresses_suite_master_payload() {
        let source = b"suite master protobuf payload";
        let mut encoder = BzEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(source).unwrap();
        let compressed = encoder.finish().unwrap();

        assert_eq!(decompress_bzip2(&compressed).unwrap(), source);
    }
}
