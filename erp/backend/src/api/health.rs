use axum::{
    extract::State,
    Json,
};
use serde_json::json;

pub async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "secure-school-erp",
        "version": "0.1.0"
    }))
}