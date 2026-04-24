use axum::{
    extract::State,
    Json,
    http::StatusCode,
};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::auth::{AuthService, Claims, AuthResponse, RegisterRequest, LoginRequest, RefreshRequest};
use crate::error::AppError;
use crate::AppState;

pub async fn register(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let state = state.read().await;
    
    if payload.email.is_empty() || payload.password.is_empty() || payload.name.is_empty() {
        return Err(AppError::BadRequest("Missing required fields".to_string()));
    }

    if payload.password.len() < 8 {
        return Err(AppError::BadRequest("Password must be at least 8 characters".to_string()));
    }

    let access_token = state.auth.generate_access_token(&payload.email, "student", "own_children_only")?;
    
    Ok(Json(AuthResponse {
        access_token,
        refresh_token: Some(state.auth.generate_refresh_token()),
        expires_in: 900,
        token_type: "Bearer".to_string(),
        requires_2fa: None,
        temp_token: None,
    }))
}

pub async fn login(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let state = state.read().await;
    
    if payload.email.is_empty() || payload.password.is_empty() {
        return Err(AppError::BadRequest("Missing email or password".to_string()));
    }

    let access_token = state.auth.generate_access_token(&payload.email, "student", "own_children_only")?;
    
    Ok(Json(AuthResponse {
        access_token,
        refresh_token: Some(state.auth.generate_refresh_token()),
        expires_in: 900,
        token_type: "Bearer".to_string(),
        requires_2fa: Some(false),
        temp_token: None,
    }))
}

pub async fn refresh(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(payload): Json<RefreshRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let state = state.read().await;
    
    let access_token = state.auth.generate_access_token("user", "student", "own_children_only")?;
    
    Ok(Json(AuthResponse {
        access_token,
        refresh_token: Some(state.auth.generate_refresh_token()),
        expires_in: 900,
        token_type: "Bearer".to_string(),
        requires_2fa: None,
        temp_token: None,
    }))
}

pub async fn logout() -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(json!({"message": "Logged out successfully"})))
}

pub async fn enable_2fa(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let state = state.read().await;
    let secret = "GENERATED_SECRET";
    
    Ok(Json(json!({
        "secret": secret,
        "qr_code": format!("otpauth://totp/SchoolERP?secret={}", secret)
    })))
}

pub async fn verify_2fa(
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(json!({"valid": true})))
}