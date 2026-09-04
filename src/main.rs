mod api;
mod cache;
mod client;
mod config;
mod crypto;
mod error;
mod proto;

use std::net::SocketAddr;
use std::sync::Arc;

use api::{routes, AppState};
use cache::{Cache, Coalescer};
use client::GarupaClient;
use config::Config;
use tracing::info;
use tracing_subscriber::EnvFilter;

fn init_tracing(level: &str) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

fn unwrap_or_exit<T>(result: Result<T, impl std::fmt::Display>, context: &str) -> T {
    result.unwrap_or_else(|error| {
        eprintln!("{context} error: {error}");
        std::process::exit(1);
    })
}

#[tokio::main]
async fn main() {
    let config = unwrap_or_exit(Config::from_env(), "configuration");

    init_tracing(&config.log_level);

    let client = unwrap_or_exit(GarupaClient::new(&config), "client");

    let state = Arc::new(AppState {
        config,
        client,
        cache: Cache::new(),
        coalescer: Coalescer::new(),
    });

    if state.config.server.enabled() {
        info!("JP server configured");
    } else {
        info!("JP server not configured");
    }

    let addr: SocketAddr = format!("{}:{}", state.config.host, state.config.port)
        .parse()
        .expect("invalid bind address");
    let listener = tokio::net::TcpListener::bind(addr).await.expect("failed to bind listener");
    info!(%addr, "penlight-dream-api listening");
    info!(prefix = %state.config.api_prefix, "API prefix");

    axum::serve(listener, routes::build(state))
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server error");
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    info!("shutting down");
}
