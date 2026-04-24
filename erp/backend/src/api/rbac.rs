use axum::{
    extract::State,
    Json,
};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::AppState;
use crate::error::AppError;

pub async fn get_roles(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let state = state.read().await;
    let roles = state.rbac.get_all_roles().await;
    
    Ok(Json(json!({
        "success": true,
        "data": roles
    })))
}

pub async fn get_permissions(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let state = state.read().await;
    let permissions = state.rbac.get_all_permissions().await;
    
    Ok(Json(json!({
        "success": true,
        "data": permissions
    })))
}

pub async fn check_access(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(payload): Json<rbac::AccessCheckRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let state = state.read().await;
    let result = state.rbac.check_access(payload).await?;
    
    Ok(json!({
        "success": true,
        "data": result
    }).into())
}