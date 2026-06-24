use axum::{Json, Router, response::IntoResponse, routing::get};
use tower_http::cors::{Any, CorsLayer};
use serde_json::json;

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        // For development, you can use Any. For production, specify exact origins.
        .allow_origin(Any) 
        .allow_methods(Any)
        .allow_headers(Any);

    // build our application with a single route
    let app = Router::new().route("/", get(input_seed)).layer(cors);

    // listen globally on port 3000
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn input_seed() -> impl IntoResponse {
    let json_response = json!({
        "input": "abcdefgh",
    });
    Json(json_response)
}