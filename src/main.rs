// #![forbid(unsafe_code)]
#![deny(
    clippy::all,
    clippy::pedantic,
    // clippy::cargo,
    clippy::nursery,
    // missing_docs,
    // rustdoc::all,
    future_incompatible
)]
// #![warn(missing_debug_implementations)]
#![allow(clippy::enum_glob_use)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::unused_async)]
#![allow(clippy::significant_drop_tightening)]

use std::sync::Arc;

use axum::{
    extract::{State, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};

use futures::future::join_all;
use tower_http::{compression::CompressionLayer, services::ServeFile, trace::TraceLayer};
use tracing::info;
use ultitato::{
    handlers::*,
    state::{AppArc, AppState},
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    info!("App opened, binding to 0.0.0.0:8080");

    let state = Arc::new(AppState::default());

    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(
            runtime_router()
                .with_state(state.clone())
                .into_make_service(),
        )
        .with_graceful_shutdown(shutdown::signal())
        .await
        .unwrap();
    join_all(state.waiting().await.drain().map(remove_waiting)).await;

    join_all(state.searching().await.drain().map(remove_searching)).await;

    info!("Shutdown finished, App closed");
}
fn runtime_router() -> Router<AppArc> {
    Router::new()
        .route_service("/", ServeFile::new("assets/html/index.html"))
        .route_service("/local", ServeFile::new("assets/html/local.html"))
        .route_service("/online", ServeFile::new("assets/html/online.html"))
        .route_service("/shared.js", ServeFile::new("assets/js/shared.js"))
        .route_service("/local.js", ServeFile::new("assets/js/local.js"))
        .route_service("/online.js", ServeFile::new("assets/js/online.js"))
        .route("/register-host", get(register_host_handler))
        .route("/register-join", get(register_join_handler))
        .route("/play", get(handle_ws))
        .fallback_service(ServeFile::new("assets/html/404.html"))
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
}
async fn handle_ws() -> impl IntoResponse {}

async fn register_host_handler(
    socket: WebSocketUpgrade,
    State(state): State<AppArc>,
) -> impl IntoResponse {
    socket.on_upgrade(|socket| handle_register_host(socket, state))
}

async fn register_join_handler(
    socket: WebSocketUpgrade,
    State(state): State<AppArc>,
) -> impl IntoResponse {
    socket.on_upgrade(|socket| handle_register_join(socket, state))
}

mod shutdown {
    use tokio::signal;
    use tracing::info;

    pub async fn signal() {
        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {
                println!();
                info!("Ctrl-C received, app shutdown commencing");
            },
            _ = terminate => {
                info!("SIGTERM received, app shutdown commencing");
            },
        }
    }
}
