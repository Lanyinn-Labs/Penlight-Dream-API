//! HTTP-level regression tests with an encrypted local upstream.
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::Duration;

use aes::cipher::{block_padding::NoPadding, BlockEncryptMut, KeyIvInit};
use axum::{routing::get, Router};
use serde_json::{json, Value};

use super::{routes, AppState};
use crate::{
    cache::{Cache, Coalescer},
    client::GarupaClient,
    config::{Config, ServerConfig},
};

struct Server {
    url: String,
    task: tokio::task::JoinHandle<()>,
}

impl Drop for Server {
    fn drop(&mut self) {
        self.task.abort();
    }
}

async fn serve(router: Router) -> Server {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let task = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    Server { url, task }
}

fn varint(mut value: u64) -> Vec<u8> {
    let mut bytes = Vec::new();
    while value >= 128 {
        bytes.push(value as u8 | 128);
        value >>= 7;
    }
    bytes.push(value as u8);
    bytes
}

fn int(tag: u64, value: u64) -> Vec<u8> {
    [varint(tag << 3), varint(value)].concat()
}
fn message(tag: u64, value: &[u8]) -> Vec<u8> {
    [varint(tag << 3 | 2), varint(value.len() as u64), value.to_vec()].concat()
}

fn encrypt(mut bytes: Vec<u8>) -> Vec<u8> {
    bytes.resize(bytes.len().div_ceil(16) * 16, 0);
    let len = bytes.len();
    cbc::Encryptor::<aes::Aes128>::new_from_slices(&[42; 16], &[24; 16])
        .unwrap()
        .encrypt_padded_mut::<NoPadding>(&mut bytes, len)
        .unwrap();
    bytes
}

fn config(base: String) -> Config {
    Config {
        server: ServerConfig {
            base,
            uid: "test".into(),
            uuid: "test-device".into(),
            client_version: "test-version".into(),
            unity_version: "test-unity".into(),
            user_agent: "test".into(),
            client_platform: "iOS".into(),
            encryption_key: vec![42; 16],
            encryption_iv: vec![24; 16],
            package_url: String::new(),
        },
        host: "127.0.0.1".into(),
        port: 0,
        api_prefix: "/api".into(),
        log_level: "error".into(),
        http_timeout_ms: 1000,
        cache_ttl_ranking_secs: 30,
        cache_ttl_master_secs: 3600,
        cache_ttl_user_secs: 300,
        version_ttl_secs: 3600,
        api_key: "test-key".into(),
    }
}

async fn request(http: &reqwest::Client, app: &Server, path: &str, status: u16) -> Value {
    let response = http
        .get(format!("{}{path}", app.url))
        .header("X-API-Key", "test-key")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), status, "{path}");
    assert_eq!(response.headers()["content-type"], "application/json");
    response.json().await.unwrap()
}

#[tokio::test]
async fn derived_routes_decode_encrypted_data_share_cache_and_validate_requests() {
    let card_calls = Arc::new(AtomicUsize::new(0));
    let character_calls = Arc::new(AtomicUsize::new(0));
    let suite_calls = Arc::new(AtomicUsize::new(0));
    let episode = [int(1, 91), message(11, "剧情".as_bytes())].concat();
    let card = [
        int(1, 1),
        int(16, 7),
        message(8, &int(1, 60)),
        message(14, &message(1, &episode)),
        message(15, &[int(1, 1), int(3, 60)].concat()),
    ]
    .concat();
    let cards = encrypt([message(1, &card), message(1, &[int(1, 2), int(16, 9)].concat())].concat());
    let characters = encrypt(
        [
            message(1, &[int(1, 7), int(9, 3)].concat()),
            message(1, &[int(1, 9), int(9, 4)].concat()),
        ]
        .concat(),
    );
    // First score omits musicId to exercise map-key fallback.
    let scores = [
        message(1, &[message(3, b"expert"), int(4, 12345), message(7, b"all_perfect")].concat()),
        message(1, &[int(2, 11), message(3, b"hard"), message(7, b"full_combo")].concat()),
    ]
    .concat();
    let suite = encrypt(message(54, &message(1, &[int(1, 11), message(2, &scores)].concat())));
    let upstream = serve(
        Router::new()
            .route(
                "/situation",
                get({
                    let calls = card_calls.clone();
                    move || {
                        calls.fetch_add(1, Ordering::SeqCst);
                        let cards = cards.clone();
                        async move { cards }
                    }
                }),
            )
            .route(
                "/character",
                get({
                    let calls = character_calls.clone();
                    move || {
                        calls.fetch_add(1, Ordering::SeqCst);
                        let characters = characters.clone();
                        async move { characters }
                    }
                }),
            )
            .route(
                "/suite/user/test",
                get({
                    let calls = suite_calls.clone();
                    move || {
                        calls.fetch_add(1, Ordering::SeqCst);
                        let suite = suite.clone();
                        async move { suite }
                    }
                }),
            ),
    )
    .await;
    let config = config(format!("{}/", upstream.url));
    let state = Arc::new(AppState {
        client: GarupaClient::new(&config).unwrap(),
        config,
        cache: Cache::new(),
        coalescer: Coalescer::new(),
    });
    let app = serve(routes::build(state)).await;
    let http = reqwest::Client::builder().timeout(Duration::from_secs(5)).build().unwrap();

    let (levels, episodes, training) = tokio::join!(
        request(&http, &app, "/api/jp/cards/1/levels", 200),
        request(&http, &app, "/api/jp/cards/1/episodes", 200),
        request(&http, &app, "/api/jp/cards/1/training", 200),
    );
    assert_eq!(levels, json!({"entries": [{"level": 60}]}));
    assert_eq!(episodes, json!({"entries": [{"episodeId": 91, "episodeName": "剧情"}]}));
    assert_eq!(training, json!({"situationId": 1, "level": 60}));
    assert_eq!(request(&http, &app, "/api/jp/cards/2/episodes", 200).await, json!({"entries": []}));
    assert_eq!(request(&http, &app, "/api/jp/cards/2/levels", 200).await, json!({"entries": []}));
    request(&http, &app, "/api/jp/cards/2/training", 404).await;
    request(&http, &app, "/api/jp/cards/999/episodes", 404).await;
    let band = request(&http, &app, "/api/jp/bands/3/cards", 200).await;
    assert_eq!(band["entries"].as_array().unwrap().len(), 1);
    assert_eq!(band["entries"][0]["situationId"], 1);
    assert_eq!(request(&http, &app, "/api/jp/bands/999/cards", 200).await, json!({"entries": []}));
    let scores = request(&http, &app, "/api/jp/user/music/11/scores", 200).await;
    assert_eq!(scores["entries"].as_array().unwrap().len(), 2);
    assert_eq!(scores["entries"][0]["musicId"], 11);
    assert_eq!(scores["entries"][0]["soloHighScore"], 12345);
    assert_eq!(
        request(&http, &app, "/api/jp/user/music/12/scores", 200).await,
        json!({"entries": []})
    );
    let status = request(&http, &app, "/api/jp/user/music/11/status?difficulty=expert", 200).await;
    assert_eq!(status["isAllPerfect"], true);
    assert_eq!(status["isFullCombo"], true);
    let list = request(&http, &app, "/api/jp/cards", 200).await;
    assert_eq!(list["entries"].as_array().unwrap().len(), 2);

    for path in [
        "cards/0/levels",
        "cards/-1/episodes",
        "cards/0/training",
        "bands/0/cards",
        "user/music/0/scores",
    ] {
        request(&http, &app, &format!("/api/jp/{path}"), 400).await;
    }
    request(&http, &app, "/api/en/cards/1/levels", 400).await;
    let unauthorized = http.get(format!("{}/api/jp/cards/1/levels", app.url)).send().await.unwrap();
    assert_eq!(unauthorized.status(), 401);
    assert_eq!(card_calls.load(Ordering::SeqCst), 1);
    assert_eq!(character_calls.load(Ordering::SeqCst), 1);
    assert_eq!(suite_calls.load(Ordering::SeqCst), 1);
    let cleared = http
        .delete(format!("{}/api/jp/cache", app.url))
        .header("X-API-Key", "test-key")
        .send()
        .await
        .unwrap();
    assert_eq!(cleared.status(), 200);
    request(&http, &app, "/api/jp/cards/1/levels", 200).await;
    assert_eq!(card_calls.load(Ordering::SeqCst), 2);
}
