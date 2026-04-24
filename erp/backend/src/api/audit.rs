use axum::{
    extract::State,
    Json,
};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::AppState;
use crate::error::AppError;
use crate::audit::{AuditQueryRequest, AuditSeverity};

pub async fn get_logs(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let state = state.read().await;
    let logs = state.audit.get_logs().await;
    
    Ok(json!({
        "success": true,
        "data": logs
    }).into())
}

pub async fn get_stats(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let state = state.read().await;
    let stats = state.audit.get_stats().await;
    
    Ok(json!({
        "success": true,
        "data": stats
    }).into())
}

pub async fn query_logs(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(payload): Json<AuditQueryRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let state = state.read().await;
    let logs = state.audit.query(payload).await?;
    
    Ok(json!({
        "success": true,
        "data": logs
    }).into())
}