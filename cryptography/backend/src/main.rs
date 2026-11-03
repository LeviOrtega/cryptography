mod cipher;
mod protocol;
mod state;
mod ws;

use std::net::SocketAddr;
use std::path::PathBuf;

use axum::{Router, routing::get};
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,tower_http=warn")),
        )
        .init();

    // --- configuration -----------------------------------------------------
    let key_raw = std::env::var("CRYPTEX_KEY")
        .expect("CRYPTEX_KEY must be set (8 letters or 8 comma-separated ints 0..26)");
    let key = match cipher::parse_key(&key_raw) {
        Ok(k) => k,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };
    let admin_token = std::env::var("CRYPTEX_ADMIN_TOKEN")
        .ok()
        .filter(|t| !t.is_empty());
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let static_dir = PathBuf::from(
        std::env::var("STATIC_DIR").unwrap_or_else(|_| "../frontend/dist".into()),
    );

    let state = state::AppState::new(key, admin_token);

    // --- routes -------------------------------------------------------------
    let mut app = Router::new()
        .route("/ws", get(ws::ws_handler))
        .route("/health", get(|| async { "ok" }));

    if static_dir.join("index.html").is_file() {
        tracing::info!(dir = %static_dir.display(), "serving frontend");
        let index = static_dir.join("index.html");
        app = app.fallback_service(ServeDir::new(&static_dir).fallback(ServeFile::new(index)));
    } else {
        tracing::warn!(
            dir = %static_dir.display(),
            "no built frontend found; only /ws and /health are served (run `npm run build` or set STATIC_DIR)"
        );
    }

    let app = app.layer(TraceLayer::new_for_http()).with_state(state);

    // --- serve --------------------------------------------------------------
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind listener");
    tracing::info!(%addr, "cryptex server listening");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(async {
        let _ = tokio::signal::ctrl_c().await;
    })
    .await
    .expect("server error");
}
