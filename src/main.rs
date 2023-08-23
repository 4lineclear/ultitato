#![forbid(unsafe_code)]
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
use axum::Router;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new()
        .nest_service("/", ServeFile::new("assets/index.html"))
        .nest_service("/play", ServeFile::new("assets/play.html"))
        .nest_service("/js", ServeDir::new("assets/js"))
        .nest_service("/css", ServeDir::new("assets/css"))
        .layer(TraceLayer::new_for_http());

    Ok(router.into())
}
