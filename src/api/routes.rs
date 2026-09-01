use axum::middleware::{from_fn, from_fn_with_state};
use axum::routing::get;
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use super::auth;
use super::handlers;
use super::server::validate_server;
use super::SharedState;

/// Builds the top-level axum router.
pub fn build(state: SharedState) -> Router {
    let api_prefix = state.config.api_prefix.clone();

    let api = Router::new()
        .route("/{server}/application", get(handlers::application))
        .route("/{server}/shops", get(handlers::shops))
        .route("/{server}/shops/{shop_id}", get(handlers::shop_single))
        .route("/{server}/cards", get(handlers::cards))
        .route("/{server}/cards/{card_id}", get(handlers::card_single))
        .route("/{server}/music", get(handlers::music_master))
        .route("/{server}/music/{music_id}", get(handlers::music_single))
        .route("/{server}/characters", get(handlers::character_master))
        .route("/{server}/characters/{character_id}/cards", get(handlers::character_cards))
        .route("/{server}/characters/{character_id}/costumes", get(handlers::character_costumes))
        .route("/{server}/characters/{character_id}", get(handlers::character_single))
        .route("/{server}/bands", get(handlers::band_master))
        .route("/{server}/bands/{band_id}/characters", get(handlers::band_characters))
        .route("/{server}/bands/{band_id}", get(handlers::band_single))
        .route("/{server}/areas", get(handlers::area_master))
        .route("/{server}/areas/{area_id}", get(handlers::area_single))
        .route("/{server}/gacha", get(handlers::gacha_master))
        .route("/{server}/gacha/{gacha_id}", get(handlers::gacha_single))
        .route("/{server}/items", get(handlers::item_master))
        .route("/{server}/items/{item_id}", get(handlers::item_single))
        .route("/{server}/skills", get(handlers::skill_master))
        .route("/{server}/skills/normalized", get(handlers::skill_master_normalized))
        .route("/{server}/skills/{skill_id}/cards", get(handlers::skill_cards))
        .route("/{server}/skills/{skill_id}", get(handlers::skill_single))
        .route("/{server}/stamps", get(handlers::stamp_master))
        .route("/{server}/stamps/{stamp_id}", get(handlers::stamp_single))
        .route("/{server}/login-bonuses", get(handlers::login_bonus_master))
        .route("/{server}/login-bonuses/{login_bonus_id}", get(handlers::login_bonus_single))
        .route("/{server}/costumes", get(handlers::costume_master))
        .route("/{server}/costumes/{costume_id}", get(handlers::costume_single))
        .route("/{server}/events", get(handlers::event_master))
        .route("/{server}/events/{event_id}/ranking", get(handlers::event_ranking))
        .route("/{server}/events/{event_id}", get(handlers::event_single))
        .route("/{server}/monthly-ranking", get(handlers::monthly_ranking_master))
        .route("/{server}/monthly-ranking/{monthly_id}", get(handlers::monthly_ranking_full))
        .route("/{server}/monthly-ranking/{monthly_id}/info", get(handlers::monthly_ranking_info))
        .route("/{server}/monthly-ranking/{monthly_id}/top", get(handlers::monthly_ranking_top))
        .route("/{server}/monthly-ranking/{monthly_id}/border", get(handlers::monthly_ranking_border))
        .route("/{server}/user/profile", get(handlers::user_profile))
        .route("/{server}/user/decks", get(handlers::user_decks))
        .route("/{server}/user/situations", get(handlers::user_situations))
        .route("/{server}/user/title", get(handlers::user_title))
        .route("/{server}/user/stamps", get(handlers::user_stamps))
        .route("/{server}/user/areas", get(handlers::user_areas))
        .route("/{server}/user/items", get(handlers::user_items))
        .route("/{server}/user/presents", get(handlers::user_presents))
        .route("/{server}/user/gacha", get(handlers::user_gacha))
        .route("/{server}/user/episodes", get(handlers::user_episodes))
        .route("/{server}/user/missions", get(handlers::user_missions))
        .route("/{server}/user/login-bonuses", get(handlers::user_login_bonuses))
        .route("/{server}/user/costumes", get(handlers::user_costumes))
        .route("/{server}/user/characters", get(handlers::user_characters))
        .route("/{server}/user/character-mission-bonuses", get(handlers::user_character_mission_bonuses))
        .route("/{server}/user/area-statuses", get(handlers::user_area_statuses))
        .route("/{server}/user/character-affinity", get(handlers::user_character_affinity))
        .route("/{server}/cache", get(handlers::cache_stats).delete(handlers::cache_clear))
        .with_state(state.clone())
        .layer(from_fn(validate_server))
        .layer(from_fn_with_state(state.clone(), auth::require_api_key));

    Router::new()
        .route("/servers", get(handlers::servers))
        .route("/health", get(handlers::health))
        .route("/version", get(handlers::version))
        .route("/image/{server}/{asset_kind}/{asset_id}", get(handlers::image_placeholder))
        .nest(&api_prefix, api)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
