use axum::{
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use std::{env, net::SocketAddr};

use tower_http::services::ServeDir;

use btc_cli::models::Fixture;
use btc_cli::analyzer::analyze;

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "ok": true }))
}

async fn analyze_handler(
    Json(payload): Json<Fixture>,
) -> Json<serde_json::Value> {
    match analyze(payload) {
        Ok(report) => Json(serde_json::to_value(report).unwrap()),
        Err(e) => Json(json!({
            "ok": false,
            "error": {
                "code": "ANALYSIS_ERROR",
                "message": e
            }
        })),
    }
}

#[tokio::main]
async fn main() {
    // Read port from environment or default to 3000
    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string());

    let addr: SocketAddr =
        format!("0.0.0.0:{}", port).parse().unwrap();

    // Build router
    let app = Router::new()
        // API routes
        .route("/api/health", get(health))
        .route("/api/analyze", post(analyze_handler))

        
        .fallback_service(ServeDir::new("static"));

    

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap();

    axum::serve(listener, app)
        .await
        .unwrap();
}