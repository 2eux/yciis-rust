pub mod error;
pub mod auth;
pub mod rbac;
pub mod audit;
pub mod api;
pub mod middleware;

pub use error::AppError;

use std::sync::Arc;
use tokio::sync::RwLock;

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