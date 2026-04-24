use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use std::net::SocketAddr;

pub mod error;
pub mod auth;
pub mod rbac;
pub mod audit;
pub mod api;
pub mod middleware;

pub use error::AppError;

use auth::AuthService;
use rbac::RbacService;
use audit::AuditService;

pub struct AppState {
    pub auth: AuthService,
    pub rbac: RbacService,
    pub audit: AuditService,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            auth: AuthService::new(),
            rbac: RbacService::new(),
            audit: AuditService::new(),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Secure School ERP starting...");
    
    let state = Arc::new(RwLock::new(AppState::new()));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .max_age_secs(3600);

    let app = Router::new()
        .route("/health", get(api::health::health_check))
        .route("/api/v1/auth/register", post(api::auth::register))
        .route("/api/v1/auth/login", post(api::auth::login))
        .route("/api/v1/auth/refresh", post(api::auth::refresh))
        .route("/api/v1/auth/logout", post(api::auth::logout))
        .route("/api/v1/auth/enable-2fa", post(api::auth::enable_2fa))
        .route("/api/v1/auth/verify-2fa", post(api::auth::verify_2fa))
        .route("/api/v1/auth/me", get(api::auth::me))
        .route("/api/v1/rbac/roles", get(api::rbac::get_roles))
        .route("/api/v1/rbac/permissions", get(api::rbac::get_permissions))
        .route("/api/v1/rbac/check", post(api::rbac::check_access))
        .route("/api/v1/audit/logs", get(api::audit::get_logs))
        .route("/api/v1/audit/stats", get(api::audit::get_stats))
        .route("/api/v1/audit/query", post(api::audit::query_logs))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .unwrap_or(3000);
    
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}