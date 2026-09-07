//! Route handlers. Each handler fetches and decrypts the corresponding Garupa
//! endpoint, decodes the protobuf, maps it to a response model, and serves it
//! through the TTL cache.

use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use axum::body::{Body, Bytes};
use axum::extract::{Path, Query, State};
use axum::http::header::{HeaderValue, CONTENT_TYPE};
use axum::response::Response;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use super::transforms::{
    filtered_entries, find_entry, flatten_map_values, flatten_nested_map_values, music_scores, normalize_music_difficulty,
    suite_character_rank_values, suite_map_values, suite_music_status_value, MUSIC_DIFFICULTIES,
};
use crate::api::models;
use crate::api::SharedState;
use crate::config::ServerConfig;
use crate::error::{AppError, AppResult};
use crate::proto::decoder::decode;
use crate::proto::garupa_schema::{
    ACTION_SET_MAP_SCHEMA, APPLICATION_SCHEMA, AREA_ITEM_MAP_SCHEMA, AREA_ITEM_SPAWN_MAP_SCHEMA, AREA_LIST_SCHEMA, BAND_LIST_SCHEMA,
    BONDS_EFFECT_MAP_SCHEMA, BONDS_MAP_SCHEMA, CHARACTER_LIST_SCHEMA, CHARACTER_SCHEMA, COSTUME_LIST_SCHEMA, DEGREE_MAP_SCHEMA,
    EVENT_TYPE_SCHEMAS, GACHA_LIST_SCHEMA, ITEM_LIST_SCHEMA, LOGIN_BONUS_LIST_SCHEMA, MASTER_EVENT_LIST_SCHEMA,
    MASTER_MONTHLY_RANKING_LIST_SCHEMA, MULTI_LIVE_DIFFICULTY_MAP_SCHEMA, MUSIC_DIFFICULTY_LIST_SCHEMA, MUSIC_LIST_SCHEMA, MUSIC_SCHEMA,
    MUSIC_SHOP_MAP_SCHEMA, SHOP_LIST_SCHEMA, SITUATION_LIST_SCHEMA, SKILL_LIST_SCHEMA, STAMP_LIST_SCHEMA, SUITE_MASTER_RESPONSE_SCHEMA,
    SUITE_USER_RESPONSE_SCHEMA, USER_AREA_LIST_SCHEMA, USER_CHARACTER_LIST_SCHEMA, USER_COSTUME_LIST_SCHEMA, USER_DECK_LIST_SCHEMA,
    USER_EPISODE_LIST_SCHEMA, USER_GACHA_LIST_SCHEMA, USER_ITEM_LIST_SCHEMA, USER_LOGIN_BONUS_LIST_SCHEMA, USER_MISSION_LIST_SCHEMA,
    USER_MONTHLY_RANKING_RANKING_RESPONSE_SCHEMA, USER_PRESENT_LIST_SCHEMA, USER_PROFILE_RESPONSE_SCHEMA, USER_SITUATION_LIST_SCHEMA,
    USER_STAMP_LIST_SCHEMA, USER_TITLE_SCHEMA, WEEKLY_MULTI_LIVE_DIFFICULTY_MAP_SCHEMA,
};
use crate::proto::schema::Schema;

/// Records the process start time for the health uptime field.
static START_TIME: OnceLock<Instant> = OnceLock::new();

/// Live JP availability snapshot, cached so frequent health probes do not
/// hammer the upstream server with one `/application` call per request.
#[derive(Clone)]
struct HealthSnapshot {
    at: Instant,
    available: bool,
    client_version: String,
}

/// How long a health snapshot is reused before the upstream is re-checked.
const HEALTH_SNAPSHOT_TTL: Duration = Duration::from_secs(10);

static HEALTH_SNAPSHOT: OnceLock<Mutex<Option<HealthSnapshot>>> = OnceLock::new();

/// Returns the JP server config, or a 404 when it is not configured.
fn jp_config(state: &SharedState) -> AppResult<&ServerConfig> {
    if state.config.server.enabled() {
        Ok(&state.config.server)
    } else {
        Err(AppError::not_found("jp server is not configured"))
    }
}

// ============================================================================
// Shared fetch helpers
// ============================================================================

/// Builds a JSON response from a pre-serialized body.
fn json_response(body: impl Into<Bytes>) -> Response {
    let mut response = Response::new(Body::from(body.into()));
    response
        .headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    response
}

/// Fetches a decoded protobuf response and returns its serialized JSON body
/// through the cache, so cache hits are served without re-parsing. The `map`
/// closure turns the decoded root into the response value. Concurrent cache
/// misses coalesce onto a single upstream call per cache window.
async fn cached_json(
    state: &SharedState,
    key: &str,
    ttl_secs: u64,
    url: &str,
    schema: &Schema,
    map: impl FnOnce(Value) -> AppResult<Value>,
) -> AppResult<Bytes> {
    if let Some(cached) = state.cache.get(key) {
        return Ok(cached);
    }
    let cfg = jp_config(state)?;
    state
        .coalescer
        .run(key, || async {
            // A previous leader may have populated the cache between our
            // initial lookup and joining this flight.
            if let Some(cached) = state.cache.get(key) {
                return Ok(cached);
            }
            let buf = state.client.fetch(cfg, url).await?;
            let root = decode(&buf, schema)?;
            let value = map(root)?;
            let body = Bytes::from(serde_json::to_vec(&value)?);
            state.cache.set(key, body.clone(), Duration::from_secs(ttl_secs));
            Ok(body)
        })
        .await
}

/// Passes a decoded response through unchanged.
fn passthrough(root: Value) -> AppResult<Value> {
    Ok(root)
}

/// Fetches a master endpoint and caches its decoded response.
async fn master_response(state: &SharedState, key: &str, url: &str, schema: &Schema) -> AppResult<Response> {
    let body = cached_json(state, key, state.config.cache_ttl_master_secs, url, schema, passthrough).await?;
    Ok(json_response(body))
}

/// Returns a cached decoded master-list root. Detail and relationship handlers
/// use the same cache key as their list endpoint, so adding derived views never
/// creates an extra upstream request.
async fn master_list_value(state: &SharedState, key: &str, url: &str, schema: &Schema) -> AppResult<Value> {
    let body = cached_json(state, key, state.config.cache_ttl_master_secs, url, schema, passthrough).await?;
    Ok(serde_json::from_slice(&body)?)
}

/// Finds a single object in an entries-wrapped master list.
async fn master_entry(
    state: &SharedState,
    key: &str,
    url: &str,
    schema: &Schema,
    id_field: &str,
    id: i64,
    resource_name: &str,
) -> AppResult<Response> {
    if id < 1 {
        return Err(AppError::bad_request(format!("{id_field} must be >= 1")));
    }
    let root = master_list_value(state, key, url, schema).await?;
    let entry = find_entry(&root, id_field, id, resource_name)?;
    Ok(json_response(entry.to_string()))
}

/// Fetches a user endpoint and caches its decoded response.
async fn user_response(state: &SharedState, key: &str, url: &str, schema: &Schema) -> AppResult<Response> {
    let body = cached_json(state, key, state.config.cache_ttl_user_secs, url, schema, passthrough).await?;
    Ok(json_response(body))
}

/// Fetches the complete suite user snapshot once and lets multiple derived
/// user endpoints share the same cached upstream response.
async fn suite_user_value(state: &SharedState, key: &str) -> AppResult<Value> {
    let cfg = jp_config(state)?;
    let body = cached_json(
        state,
        key,
        state.config.cache_ttl_user_secs,
        &state.client.suite_user_url(cfg),
        &SUITE_USER_RESPONSE_SCHEMA,
        passthrough,
    )
    .await?;
    Ok(serde_json::from_slice(&body)?)
}

/// Fetches a map-shaped master endpoint and exposes its values as a list. The
/// upstream map key is copied into the requested identifier field when the
/// nested object does not contain one.
async fn map_list_body(state: &SharedState, key: &str, url: &str, schema: &Schema, key_name: Option<&str>) -> AppResult<Bytes> {
    let body = cached_json(state, key, state.config.cache_ttl_master_secs, url, schema, |root| {
        Ok(flatten_map_values(&root, key_name, false))
    })
    .await?;
    Ok(body)
}

macro_rules! master_handlers {
    ($($(#[$meta:meta])* $name:ident => ($key:literal, $url:ident, $schema:ident);)+) => {
        $(
            $(#[$meta])*
            pub async fn $name(State(state): State<SharedState>) -> AppResult<Response> {
                let cfg = jp_config(&state)?;
                master_response(&state, $key, &state.client.$url(cfg), &$schema).await
            }
        )+
    };
}

/// Declare both views together so they cannot disagree on decoding or cache keys.
macro_rules! map_handlers {
    ($($(#[$meta:meta])* $name:ident, $detail:ident => ($key:literal, $url:ident, $schema:ident, $key_field:literal, $id_type:ty);)+) => {
        $(
            $(#[$meta])*
            pub async fn $name(State(state): State<SharedState>) -> AppResult<Response> {
                let cfg = jp_config(&state)?;
                Ok(json_response(map_list_body(&state, $key, &state.client.$url(cfg), &$schema, Some($key_field)).await?))
            }

            pub async fn $detail(
                State(state): State<SharedState>,
                Path((_server, id)): Path<(String, String)>,
            ) -> AppResult<Response> {
                let id = id.parse::<$id_type>()
                    .map_err(|_| AppError::bad_request(concat!("invalid ", $key_field)))?;
                let identifier = json!(id);
                if identifier.as_i64().is_some_and(|id| id < 1) {
                    return Err(AppError::bad_request(concat!($key_field, " must be >= 1")));
                }
                let cfg = jp_config(&state)?;
                let body = map_list_body(&state, $key, &state.client.$url(cfg), &$schema, Some($key_field)).await?;
                let root: Value = serde_json::from_slice(&body)?;
                let entry = root.get("entries").and_then(Value::as_array)
                    .and_then(|entries| entries.iter().find(|entry| entry.get($key_field) == Some(&identifier)))
                    .ok_or_else(|| AppError::not_found(format!("{} {} not found", $key, id)))?;
                Ok(json_response(entry.to_string()))
            }
        )+
    };
}

macro_rules! user_handlers {
    ($($(#[$meta:meta])* $name:ident => ($key:literal, $url:ident, $schema:ident);)+) => {
        $(
            $(#[$meta])*
            pub async fn $name(State(state): State<SharedState>) -> AppResult<Response> {
                let cfg = jp_config(&state)?;
                let key = format!("{}:{}", $key, cfg.uid);
                user_response(&state, &key, &state.client.$url(cfg), &$schema).await
            }
        )+
    };
}

macro_rules! master_entry_handlers {
    ($($(#[$meta:meta])* $name:ident($id:ident) => ($key:literal, $url:ident, $schema:ident, $id_field:literal, $resource:literal);)+) => {
        $(
            $(#[$meta])*
            pub async fn $name(
                State(state): State<SharedState>,
                Path((_server, $id)): Path<(String, i64)>,
            ) -> AppResult<Response> {
                let cfg = jp_config(&state)?;
                master_entry(&state, $key, &state.client.$url(cfg), &$schema, $id_field, $id, $resource).await
            }
        )+
    };
}

macro_rules! filtered_master_handlers {
    ($($(#[$meta:meta])* $name:ident($id:ident) => ($key:literal, $url:ident, $schema:ident, $id_field:literal, $filter_field:literal);)+) => {
        $(
            $(#[$meta])*
            pub async fn $name(
                State(state): State<SharedState>,
                Path((_server, $id)): Path<(String, i64)>,
            ) -> AppResult<Response> {
                if $id < 1 {
                    return Err(AppError::bad_request(concat!($id_field, " must be >= 1")));
                }
                let cfg = jp_config(&state)?;
                let root = master_list_value(&state, $key, &state.client.$url(cfg), &$schema).await?;
                Ok(json_response(filtered_entries(&root, |entry| {
                    entry.get($filter_field).and_then(Value::as_i64) == Some($id)
                }).to_string()))
            }
        )+
    };
}

// ============================================================================
// Server metadata
// ============================================================================

/// GET /servers — lists the configured server, omitting credentials and the
/// player UID so the endpoint can stay unauthenticated.
pub async fn servers(State(state): State<SharedState>) -> Json<Value> {
    let cfg = &state.config.server;
    if !cfg.enabled() {
        return Json(Value::Array(Vec::new()));
    }
    Json(Value::Array(vec![json!({
        "index": 0,
        "name": "jp",
        "base": cfg.base,
    })]))
}

/// GET /health — process health envelope plus live JP availability check,
/// with the upstream probe cached for a short window.
pub async fn health(State(state): State<SharedState>) -> Json<Value> {
    let cfg = &state.config.server;
    let (available, client_version) = {
        // Clone the snapshot out so the mutex guard is dropped before any
        // `.await` below; a `MutexGuard` is not `Send`.
        let snapshot = HEALTH_SNAPSHOT.get_or_init(|| Mutex::new(None)).lock().unwrap().clone();
        match snapshot {
            Some(s) if s.at.elapsed() < HEALTH_SNAPSHOT_TTL => (s.available, s.client_version),
            _ => {
                let available = if cfg.enabled() {
                    state.client.check_health(cfg).await
                } else {
                    false
                };
                let client_version = if cfg.enabled() {
                    state.client.client_version(cfg).await
                } else {
                    String::new()
                };
                if let Ok(mut guard) = HEALTH_SNAPSHOT.get_or_init(|| Mutex::new(None)).lock() {
                    *guard = Some(HealthSnapshot {
                        at: Instant::now(),
                        available,
                        client_version: client_version.clone(),
                    });
                }
                (available, client_version)
            }
        }
    };
    let uptime = START_TIME.get_or_init(Instant::now).elapsed().as_secs();
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "uptime_secs": uptime,
        "jp": { "available": available, "clientVersion": client_version },
    }))
}

/// GET /version — the current auto-detected client version.
pub async fn version(State(state): State<SharedState>) -> Json<Value> {
    let cfg = &state.config.server;
    let client_version = if cfg.enabled() {
        state.client.client_version(cfg).await
    } else {
        String::new()
    };
    Json(json!({ "jp": { "clientVersion": client_version } }))
}

// ============================================================================
// Monthly ranking
// ============================================================================

/// Fetches and caches the normalized monthly ranking period list.
async fn fetch_monthly_master_body(state: &SharedState) -> AppResult<Bytes> {
    let cfg = jp_config(state)?;
    cached_json(
        state,
        "monthly-master",
        state.config.cache_ttl_master_secs,
        &state.client.monthly_ranking_master_url(cfg),
        &MASTER_MONTHLY_RANKING_LIST_SCHEMA,
        |root| Ok(json!({ "entries": serde_json::to_value(models::monthly_ranking_list(&root))? })),
    )
    .await
}

/// GET /api/{server}/monthly-ranking — master list of monthly ranking periods.
pub async fn monthly_ranking_master(State(state): State<SharedState>) -> AppResult<Response> {
    let body = fetch_monthly_master_body(&state).await?;
    Ok(json_response(body))
}

/// GET /api/{server}/monthly-ranking/{monthly_id}/info — one master period.
pub async fn monthly_ranking_info(
    State(state): State<SharedState>,
    Path((_server, monthly_id)): Path<(String, i64)>,
) -> AppResult<Response> {
    if monthly_id < 1 {
        return Err(AppError::bad_request("monthlyRankingId must be >= 1"));
    }
    let body = fetch_monthly_master_body(&state).await?;
    let root: Value = serde_json::from_slice(&body)?;
    let entry = root
        .get("entries")
        .and_then(Value::as_array)
        .and_then(|entries| {
            entries
                .iter()
                .find(|entry| entry.get("monthlyRankingId").and_then(Value::as_i64) == Some(monthly_id))
        })
        .cloned()
        .ok_or_else(|| AppError::not_found(format!("monthly ranking {monthly_id} not found")))?;
    Ok(json_response(entry.to_string()))
}

/// Fetches and caches the serialized monthly ranking report for a period.
async fn fetch_monthly_body(state: &SharedState, monthly_id: i64) -> AppResult<Bytes> {
    let cfg = jp_config(state)?;
    if monthly_id < 1 {
        return Err(AppError::bad_request("monthlyId must be >= 1"));
    }
    let key = format!("monthly:{monthly_id}");
    cached_json(
        state,
        &key,
        state.config.cache_ttl_ranking_secs,
        &state.client.monthly_ranking_url(cfg, monthly_id),
        &USER_MONTHLY_RANKING_RANKING_RESPONSE_SCHEMA,
        |root| Ok(serde_json::to_value(models::monthly_ranking_report(&root))?),
    )
    .await
}

/// Fetches the monthly ranking report as a value for sub-endpoint extraction.
async fn fetch_monthly_ranking_value(state: &SharedState, monthly_id: i64) -> AppResult<Value> {
    let body = fetch_monthly_body(state, monthly_id).await?;
    Ok(serde_json::from_slice(&body)?)
}

/// GET /api/{server}/monthly-ranking/{monthly_id} — near/top/border users.
pub async fn monthly_ranking_full(
    State(state): State<SharedState>,
    Path((_server, monthly_id)): Path<(String, i64)>,
) -> AppResult<Response> {
    let body = fetch_monthly_body(&state, monthly_id).await?;
    Ok(json_response(body))
}

/// GET /api/{server}/monthly-ranking/{monthly_id}/top — top users only.
pub async fn monthly_ranking_top(
    State(state): State<SharedState>,
    Path((_server, monthly_id)): Path<(String, i64)>,
) -> AppResult<Json<Value>> {
    let full = fetch_monthly_ranking_value(&state, monthly_id).await?;
    let users = full
        .get("monthlyRankingPointTopUsers")
        .cloned()
        .unwrap_or_else(|| Value::Array(Vec::new()));
    Ok(Json(json!({ "users": users })))
}

/// GET /api/{server}/monthly-ranking/{monthly_id}/border — border users only.
pub async fn monthly_ranking_border(
    State(state): State<SharedState>,
    Path((_server, monthly_id)): Path<(String, i64)>,
) -> AppResult<Json<Value>> {
    let full = fetch_monthly_ranking_value(&state, monthly_id).await?;
    let users = full
        .get("monthlyRankingPointBorderUsers")
        .cloned()
        .unwrap_or_else(|| Value::Array(Vec::new()));
    Ok(Json(json!({ "users": users })))
}

// ============================================================================
// Event
// ============================================================================

/// Fetches and caches the normalized event master list.
async fn fetch_event_master_body(state: &SharedState) -> AppResult<Bytes> {
    let cfg = jp_config(state)?;
    cached_json(
        state,
        "event-master",
        state.config.cache_ttl_master_secs,
        &state.client.event_master_url(cfg),
        &MASTER_EVENT_LIST_SCHEMA,
        |root| Ok(json!({ "entries": serde_json::to_value(models::event_list(&root))? })),
    )
    .await
}

/// GET /api/{server}/events — master list of events.
pub async fn event_master(State(state): State<SharedState>) -> AppResult<Response> {
    let body = fetch_event_master_body(&state).await?;
    Ok(json_response(body))
}

/// GET /api/{server}/events/{event_id} — one event from the cached master list.
pub async fn event_single(State(state): State<SharedState>, Path((_server, event_id)): Path<(String, i64)>) -> AppResult<Response> {
    if event_id < 1 {
        return Err(AppError::bad_request("eventId must be >= 1"));
    }
    let events = fetch_event_master_value(&state).await?;
    let entry = events
        .as_array()
        .and_then(|entries| {
            entries
                .iter()
                .find(|entry| entry.get("eventId").and_then(Value::as_i64) == Some(event_id))
        })
        .cloned()
        .ok_or_else(|| AppError::not_found(format!("event {event_id} not found")))?;
    Ok(json_response(entry.to_string()))
}

/// Fetches the event master list as a bare entries array for type resolution.
async fn fetch_event_master_value(state: &SharedState) -> AppResult<Value> {
    let body = fetch_event_master_body(state).await?;
    let wrapped: Value = serde_json::from_slice(&body)?;
    Ok(wrapped.get("entries").cloned().unwrap_or_else(|| Value::Array(Vec::new())))
}

#[derive(Debug, Deserialize)]
pub struct EventRankingQuery {
    /// Protobuf event type such as "medley" or "versus". When omitted, it is
    /// resolved from the event master list.
    pub r#type: Option<String>,
    /// Optional music ID for challenge/versus per-song sub-rankings.
    pub mid: Option<i64>,
}

/// GET /api/{server}/events/{event_id}/ranking — event ranking for a specific event.
pub async fn event_ranking(
    State(state): State<SharedState>,
    Path((_server, event_id)): Path<(String, i64)>,
    Query(query): Query<EventRankingQuery>,
) -> AppResult<Response> {
    let cfg = jp_config(&state)?;
    if event_id < 1 {
        return Err(AppError::bad_request("eventId must be >= 1"));
    }

    let event_type = match &query.r#type {
        Some(t) => t.clone(),
        None => {
            let events = fetch_event_master_value(&state).await?;
            let entry = events
                .as_array()
                .and_then(|arr| arr.iter().find(|e| e.get("eventId").and_then(Value::as_i64) == Some(event_id)));
            match entry.and_then(|e| e.get("eventType").and_then(Value::as_str)) {
                Some(t) => t.to_string(),
                None => return Err(AppError::not_found(format!("event {event_id} not found"))),
            }
        }
    };

    let schema = match EVENT_TYPE_SCHEMAS.iter().find(|entry| entry.0 == event_type.as_str()) {
        Some(entry) => entry.1,
        None => {
            let supported: Vec<&str> = EVENT_TYPE_SCHEMAS.iter().map(|entry| entry.0).collect();
            return Err(AppError::bad_request(format!(
                "unsupported event type \"{event_type}\", supported: {}",
                supported.join(", ")
            )));
        }
    };

    let key = format!("event-ranking:{event_id}:{event_type}:{}", query.mid.unwrap_or(0));
    let url = state.client.event_ranking_url(cfg, event_id, &event_type, query.mid);
    let body = cached_json(&state, &key, state.config.cache_ttl_ranking_secs, &url, schema, |root| {
        Ok(serde_json::to_value(models::event_ranking_report(&root, &event_type))?)
    })
    .await?;
    Ok(json_response(body))
}

// ============================================================================
// Application
// ============================================================================

master_handlers! {
    /// GET /api/{server}/application — app version and maintenance status.
    application => ("application", application_url, APPLICATION_SCHEMA);
    /// GET /api/{server}/music — music master list.
    music_master => ("music-master", music_master_url, MUSIC_LIST_SCHEMA);
    /// GET /api/{server}/music-difficulties — chart and score thresholds.
    music_difficulty_master => ("music-difficulty-master", music_difficulty_master_url, MUSIC_DIFFICULTY_LIST_SCHEMA);
    /// GET /api/{server}/master-suite — selected fields from the full snapshot.
    master_suite => ("suite-master", suite_master_url, SUITE_MASTER_RESPONSE_SCHEMA);
    /// GET /api/{server}/characters — character master list.
    character_master => ("character-master", character_master_url, CHARACTER_LIST_SCHEMA);
    /// GET /api/{server}/bands — band master list.
    band_master => ("band-master", band_master_url, BAND_LIST_SCHEMA);
    /// GET /api/{server}/areas — area master list.
    area_master => ("area-master", area_master_url, AREA_LIST_SCHEMA);
    /// GET /api/{server}/gacha — gacha master list.
    gacha_master => ("gacha-master", gacha_master_url, GACHA_LIST_SCHEMA);
    /// GET /api/{server}/items — item master list.
    item_master => ("item-master", item_master_url, ITEM_LIST_SCHEMA);
    /// GET /api/{server}/skills — skill master list.
    skill_master => ("skill-master", skill_master_url, SKILL_LIST_SCHEMA);
    /// GET /api/{server}/stamps — stamp master list.
    stamp_master => ("stamp-master", stamp_master_url, STAMP_LIST_SCHEMA);
    /// GET /api/{server}/login-bonuses — login bonus master list.
    login_bonus_master => ("loginbonus-master", login_bonus_master_url, LOGIN_BONUS_LIST_SCHEMA);
    /// GET /api/{server}/costumes — costume master list.
    costume_master => ("costume-master", costume_master_url, COSTUME_LIST_SCHEMA);
    /// GET /api/{server}/shops — shop master list.
    shops => ("shop-master", shop_url, SHOP_LIST_SCHEMA);
    /// GET /api/{server}/cards — card (situation) master list.
    cards => ("situation-master", situation_master_url, SITUATION_LIST_SCHEMA);
}

master_entry_handlers! {
    /// GET /api/{server}/bands/{band_id} — one band.
    band_single(band_id) => ("band-master", band_master_url, BAND_LIST_SCHEMA, "bandId", "band");
    /// GET /api/{server}/areas/{area_id} — one area.
    area_single(area_id) => ("area-master", area_master_url, AREA_LIST_SCHEMA, "areaId", "area");
    /// GET /api/{server}/gacha/{gacha_id} — one gacha.
    gacha_single(gacha_id) => ("gacha-master", gacha_master_url, GACHA_LIST_SCHEMA, "gachaId", "gacha");
    /// GET /api/{server}/items/{item_id} — one item.
    item_single(item_id) => ("item-master", item_master_url, ITEM_LIST_SCHEMA, "itemId", "item");
    /// GET /api/{server}/stamps/{stamp_id} — one stamp.
    stamp_single(stamp_id) => ("stamp-master", stamp_master_url, STAMP_LIST_SCHEMA, "stampId", "stamp");
    /// GET /api/{server}/login-bonuses/{login_bonus_id} — one campaign.
    login_bonus_single(login_bonus_id) => ("loginbonus-master", login_bonus_master_url, LOGIN_BONUS_LIST_SCHEMA, "loginBonusId", "login bonus");
    /// GET /api/{server}/costumes/{costume_id} — one costume.
    costume_single(costume_id) => ("costume-master", costume_master_url, COSTUME_LIST_SCHEMA, "costumeId", "costume");
    /// GET /api/{server}/shops/{shop_id} — one shop.
    shop_single(shop_id) => ("shop-master", shop_url, SHOP_LIST_SCHEMA, "shopId", "shop");
    /// GET /api/{server}/cards/{card_id} — one card.
    card_single(card_id) => ("situation-master", situation_master_url, SITUATION_LIST_SCHEMA, "situationId", "card");
}

filtered_master_handlers! {
    /// GET /api/{server}/music/{music_id}/difficulties — chart metadata.
    music_difficulties(music_id) => ("music-difficulty-master", music_difficulty_master_url, MUSIC_DIFFICULTY_LIST_SCHEMA, "musicId", "musicId");
    /// GET /api/{server}/characters/{character_id}/cards — character cards.
    character_cards(character_id) => ("situation-master", situation_master_url, SITUATION_LIST_SCHEMA, "characterId", "characterIndex");
    /// GET /api/{server}/characters/{character_id}/costumes — character costumes.
    character_costumes(character_id) => ("costume-master", costume_master_url, COSTUME_LIST_SCHEMA, "characterId", "characterId");
    /// GET /api/{server}/bands/{band_id}/characters — band members.
    band_characters(band_id) => ("character-master", character_master_url, CHARACTER_LIST_SCHEMA, "bandId", "bandId");
}

/// Card sections reuse the situation list cache and preserve upstream fields.
async fn card_section(state: &SharedState, card_id: i64, section: &str) -> AppResult<Response> {
    if card_id < 1 {
        return Err(AppError::bad_request("cardId must be >= 1"));
    }
    let cfg = jp_config(state)?;
    let root = master_list_value(
        state,
        "situation-master",
        &state.client.situation_master_url(cfg),
        &SITUATION_LIST_SCHEMA,
    )
    .await?;
    let card = find_entry(&root, "situationId", card_id, "card")?;
    let value = match section {
        "levels" => json!({ "entries": card.get("levels").cloned().unwrap_or_else(|| json!([])) }),
        "episodes" => {
            json!({ "entries": card.get("episodes").and_then(|episodes| episodes.get("entries")).cloned().unwrap_or_else(|| json!([])) })
        }
        _ => card
            .get("training")
            .cloned()
            .ok_or_else(|| AppError::not_found(format!("card {card_id} has no training data")))?,
    };
    Ok(json_response(value.to_string()))
}

pub async fn card_levels(State(state): State<SharedState>, Path((_server, card_id)): Path<(String, i64)>) -> AppResult<Response> {
    card_section(&state, card_id, "levels").await
}

pub async fn card_episodes(State(state): State<SharedState>, Path((_server, card_id)): Path<(String, i64)>) -> AppResult<Response> {
    card_section(&state, card_id, "episodes").await
}

pub async fn card_training(State(state): State<SharedState>, Path((_server, card_id)): Path<(String, i64)>) -> AppResult<Response> {
    card_section(&state, card_id, "training").await
}

/// Join character membership to cards; character IDs are not assumed to be
/// contiguous or to encode the band ID.
pub async fn band_cards(State(state): State<SharedState>, Path((_server, band_id)): Path<(String, i64)>) -> AppResult<Response> {
    if band_id < 1 {
        return Err(AppError::bad_request("bandId must be >= 1"));
    }
    let cfg = jp_config(&state)?;
    let characters = master_list_value(
        &state,
        "character-master",
        &state.client.character_master_url(cfg),
        &CHARACTER_LIST_SCHEMA,
    )
    .await?;
    let members: HashSet<i64> = characters
        .get("entries")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|entry| entry.get("bandId").and_then(Value::as_i64) == Some(band_id))
        .filter_map(|entry| entry.get("characterId").and_then(Value::as_i64))
        .collect();
    if members.is_empty() {
        return Ok(json_response("{\"entries\":[]}"));
    }
    let cards = master_list_value(
        &state,
        "situation-master",
        &state.client.situation_master_url(cfg),
        &SITUATION_LIST_SCHEMA,
    )
    .await?;
    Ok(json_response(
        filtered_entries(&cards, |card| {
            card.get("characterIndex")
                .and_then(Value::as_i64)
                .is_some_and(|id| members.contains(&id))
        })
        .to_string(),
    ))
}

// ============================================================================
// Master data
// ============================================================================

/// GET /api/{server}/music/{music_id} — a single music master entry. The
/// upstream `music/{id}` response is a bare object (verified by live probe),
/// not the `entries` wrapper the list endpoint uses.
pub async fn music_single(State(state): State<SharedState>, Path((_server, music_id)): Path<(String, i64)>) -> AppResult<Response> {
    let cfg = jp_config(&state)?;
    if music_id < 1 {
        return Err(AppError::bad_request("musicId must be >= 1"));
    }
    let key = format!("music-single:{music_id}");
    master_response(&state, &key, &state.client.music_single_url(cfg, music_id), &MUSIC_SCHEMA).await
}

map_handlers! {
    /// GET /api/{server}/multi-live-difficulties — regular live rules.
    multi_live_difficulty_master, multi_live_difficulty_single => ("multi-live-difficulty-master", multi_live_difficulty_master_url, MULTI_LIVE_DIFFICULTY_MAP_SCHEMA, "id", i64);
    /// GET /api/{server}/weekly-multi-live-difficulties — weekly live rules.
    weekly_multi_live_difficulty_master, weekly_multi_live_difficulty_single => ("weekly-multi-live-difficulty-master", weekly_multi_live_difficulty_master_url, WEEKLY_MULTI_LIVE_DIFFICULTY_MAP_SCHEMA, "id", i64);
    /// GET /api/{server}/area-items — area item metadata.
    area_item_master, area_item_single => ("area-item-master", area_item_master_url, AREA_ITEM_MAP_SCHEMA, "areaItemId", i64);
    /// GET /api/{server}/area-item-spawns — area item placement points.
    area_item_spawn_master, area_item_spawn_single => ("area-item-spawn-master", area_item_spawn_master_url, AREA_ITEM_SPAWN_MAP_SCHEMA, "spawnPoint", String);
    /// GET /api/{server}/bonds — character bond definitions.
    bonds_master, bonds_single => ("bonds-master", bonds_master_url, BONDS_MAP_SCHEMA, "bondsId", i64);
    /// GET /api/{server}/bond-effects — bonuses granted by bonds.
    bonds_effect_master, bonds_effect_single => ("bonds-effect-master", bonds_effect_master_url, BONDS_EFFECT_MAP_SCHEMA, "bondsEffectId", i64);
    /// GET /api/{server}/action-sets — area action sets.
    action_set_master, action_set_single => ("action-set-master", action_set_master_url, ACTION_SET_MAP_SCHEMA, "actionSetId", i64);
    /// GET /api/{server}/music-shops — song exchange entries.
    music_shop_master, music_shop_single => ("music-shop-master", music_shop_master_url, MUSIC_SHOP_MAP_SCHEMA, "musicShopId", i64);
    /// GET /api/{server}/degrees — profile degree/badge metadata.
    degree_master, degree_single => ("degree-master", degree_master_url, DEGREE_MAP_SCHEMA, "degreeId", i64);
}

/// GET /api/{server}/characters/{character_id} — a single character master
/// entry. The upstream `character/{id}` response is a bare object (verified by
/// live probe), not the `entries` wrapper the list endpoint uses.
pub async fn character_single(State(state): State<SharedState>, Path((_server, character_id)): Path<(String, i64)>) -> AppResult<Response> {
    let cfg = jp_config(&state)?;
    if character_id < 1 {
        return Err(AppError::bad_request("characterId must be >= 1"));
    }
    let key = format!("character-single:{character_id}");
    master_response(
        &state,
        &key,
        &state.client.character_single_url(cfg, character_id),
        &CHARACTER_SCHEMA,
    )
    .await
}

/// GET /api/{server}/skills/normalized — skills grouped by ID with an ordered
/// level list and an explicit duration field. The raw endpoint remains
/// unchanged for backwards compatibility.
pub async fn skill_master_normalized(State(state): State<SharedState>) -> AppResult<Response> {
    let cfg = jp_config(&state)?;
    let root = master_list_value(&state, "skill-master", &state.client.skill_master_url(cfg), &SKILL_LIST_SCHEMA).await?;
    Ok(json_response(
        json!({ "entries": serde_json::to_value(models::skill_list(&root))? }).to_string(),
    ))
}

/// GET /api/{server}/skills/{skill_id} — one normalized skill with all levels.
pub async fn skill_single(State(state): State<SharedState>, Path((_server, skill_id)): Path<(String, i64)>) -> AppResult<Response> {
    if skill_id < 1 {
        return Err(AppError::bad_request("skillId must be >= 1"));
    }
    let cfg = jp_config(&state)?;
    let root = master_list_value(&state, "skill-master", &state.client.skill_master_url(cfg), &SKILL_LIST_SCHEMA).await?;
    let skill = models::skill_list(&root)
        .into_iter()
        .find(|skill| skill.skill_id == skill_id)
        .ok_or_else(|| AppError::not_found(format!("skill {skill_id} not found")))?;
    Ok(json_response(serde_json::to_string(&skill)?))
}

/// GET /api/{server}/skills/{skill_id}/cards — cards using a skill.
pub async fn skill_cards(State(state): State<SharedState>, Path((_server, skill_id)): Path<(String, i64)>) -> AppResult<Response> {
    if skill_id < 1 {
        return Err(AppError::bad_request("skillId must be >= 1"));
    }
    let cfg = jp_config(&state)?;
    let root = master_list_value(
        &state,
        "situation-master",
        &state.client.situation_master_url(cfg),
        &SITUATION_LIST_SCHEMA,
    )
    .await?;
    Ok(json_response(
        filtered_entries(&root, |entry| {
            entry.get("skillId").and_then(Value::as_i64) == Some(skill_id)
                || entry.get("skillId2").and_then(Value::as_i64) == Some(skill_id)
        })
        .to_string(),
    ))
}

// ============================================================================
// User data
// ============================================================================

user_handlers! {
    /// GET /api/{server}/user/profile — profile and stats.
    user_profile => ("user-profile", user_profile_url, USER_PROFILE_RESPONSE_SCHEMA);
    /// GET /api/{server}/user/decks — decks.
    user_decks => ("user-decks", user_deck_url, USER_DECK_LIST_SCHEMA);
    /// GET /api/{server}/user/situations — owned cards.
    user_situations => ("user-situations", user_situation_url, USER_SITUATION_LIST_SCHEMA);
    /// GET /api/{server}/user/title — equipped title.
    user_title => ("user-title", user_title_url, USER_TITLE_SCHEMA);
    /// GET /api/{server}/user/stamps — owned stamps.
    user_stamps => ("user-stamps", user_stamp_url, USER_STAMP_LIST_SCHEMA);
    /// GET /api/{server}/user/items — item balances.
    user_items => ("user-items", user_item_url, USER_ITEM_LIST_SCHEMA);
    /// GET /api/{server}/user/presents — presents and box information.
    user_presents => ("user-presents", user_present_url, USER_PRESENT_LIST_SCHEMA);
    /// GET /api/{server}/user/gacha — gacha records.
    user_gacha => ("user-gacha", user_gacha_url, USER_GACHA_LIST_SCHEMA);
    /// GET /api/{server}/user/episodes — unlocked episodes.
    user_episodes => ("user-episodes", user_episode_url, USER_EPISODE_LIST_SCHEMA);
    /// GET /api/{server}/user/missions — mission progress.
    user_missions => ("user-missions", user_mission_url, USER_MISSION_LIST_SCHEMA);
    /// GET /api/{server}/user/login-bonuses — login bonus progress.
    user_login_bonuses => ("user-login-bonuses", user_login_bonus_url, USER_LOGIN_BONUS_LIST_SCHEMA);
    /// GET /api/{server}/user/costumes — owned costumes.
    user_costumes => ("user-costumes", user_costume_url, USER_COSTUME_LIST_SCHEMA);
    /// GET /api/{server}/user/area-statuses — raw area status records.
    user_area_statuses => ("user-area-statuses", user_area_url, USER_AREA_LIST_SCHEMA);
    /// GET /api/{server}/user/character-affinity — character affinity records.
    user_character_affinity => ("user-character-affinity", user_character_url, USER_CHARACTER_LIST_SCHEMA);
}

/// GET /api/{server}/user/areas — enabled area items with category and level.
pub async fn user_areas(State(state): State<SharedState>) -> AppResult<Response> {
    let cfg = jp_config(&state)?;
    let root = suite_user_value(&state, &format!("suite-user:{}", cfg.uid)).await?;
    Ok(json_response(suite_map_values(&root, "userAreaItemMap", None).to_string()))
}

/// GET /api/{server}/user/music-scores — all recorded per-song scores.
pub async fn user_music_scores(State(state): State<SharedState>) -> AppResult<Response> {
    let cfg = jp_config(&state)?;
    let root = suite_user_value(&state, &format!("suite-user:{}", cfg.uid)).await?;
    let entries = flatten_nested_map_values(&root, "userMusicScoreMap", "musicId");
    Ok(json_response(json!({ "entries": entries }).to_string()))
}

/// All recorded difficulties for one song, preserving the source order.
pub async fn user_music_scores_single(
    State(state): State<SharedState>,
    Path((_server, music_id)): Path<(String, i64)>,
) -> AppResult<Response> {
    if music_id < 1 {
        return Err(AppError::bad_request("musicId must be >= 1"));
    }
    let cfg = jp_config(&state)?;
    let root = suite_user_value(&state, &format!("suite-user:{}", cfg.uid)).await?;
    let entries: Vec<Value> = music_scores(&root, music_id)
        .map(|score| {
            let mut score = score.clone();
            if let Some(object) = score.as_object_mut() {
                object.entry("musicId").or_insert_with(|| json!(music_id));
            }
            score
        })
        .collect();
    Ok(json_response(json!({ "entries": entries }).to_string()))
}

#[derive(Debug, Deserialize)]
pub struct UserMusicStatusQuery {
    /// One of `easy`, `normal`, `hard`, `expert`, or `special`.
    pub difficulty: Option<String>,
}

/// GET /api/{server}/user/music/{music_id}/status — clear/FC/AP status for a
/// particular song difficulty.
pub async fn user_music_status(
    State(state): State<SharedState>,
    Path((_server, music_id)): Path<(String, i64)>,
    Query(query): Query<UserMusicStatusQuery>,
) -> AppResult<Response> {
    if music_id < 1 {
        return Err(AppError::bad_request("musicId must be >= 1"));
    }

    let raw_difficulty = query
        .difficulty
        .as_deref()
        .ok_or_else(|| AppError::bad_request("difficulty is required"))?;
    let difficulty = normalize_music_difficulty(raw_difficulty).ok_or_else(|| {
        AppError::bad_request(format!(
            "unsupported difficulty \"{raw_difficulty}\", supported: {}",
            MUSIC_DIFFICULTIES.join(", ")
        ))
    })?;

    let cfg = jp_config(&state)?;
    let root = suite_user_value(&state, &format!("suite-user:{}", cfg.uid)).await?;
    Ok(json_response(suite_music_status_value(&root, music_id, difficulty).to_string()))
}

/// GET /api/{server}/user/music-clear-info — aggregate clear/FC/AP counts by
/// difficulty from the same official suite snapshot.
pub async fn user_music_clear_info(State(state): State<SharedState>) -> AppResult<Response> {
    let cfg = jp_config(&state)?;
    let root = suite_user_value(&state, &format!("suite-user:{}", cfg.uid)).await?;
    Ok(json_response(
        suite_map_values(&root, "userMusicClearInfoMap", Some("difficulty")).to_string(),
    ))
}

/// GET /api/{server}/user/characters — character rank, experience and potential.
pub async fn user_characters(State(state): State<SharedState>) -> AppResult<Response> {
    let cfg = jp_config(&state)?;
    let root = suite_user_value(&state, &format!("suite-user:{}", cfg.uid)).await?;
    Ok(json_response(suite_character_rank_values(&root).to_string()))
}

/// GET /api/{server}/user/character-mission-bonuses — character mission bonuses.
pub async fn user_character_mission_bonuses(State(state): State<SharedState>) -> AppResult<Response> {
    let cfg = jp_config(&state)?;
    let root = suite_user_value(&state, &format!("suite-user:{}", cfg.uid)).await?;
    let entries = flatten_nested_map_values(&root, "userCharacterMissionBonusMap", "characterId");
    Ok(json_response(json!({ "entries": entries }).to_string()))
}

// ============================================================================
// Cache
// ============================================================================

/// GET /api/{server}/cache — cache diagnostics.
pub async fn cache_stats(State(state): State<SharedState>) -> Json<Value> {
    Json(json!({ "entries": state.cache.len() }))
}

/// DELETE /api/{server}/cache — clears the response cache.
pub async fn cache_clear(State(state): State<SharedState>) -> Json<Value> {
    state.cache.clear();
    Json(json!({ "cleared": true }))
}
