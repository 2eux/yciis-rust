use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use crate::auth::AuthService;

pub mod auth {
    use super::*;

    pub async fn check_auth(
        request: Request,
        next: Next,
    ) -> Result<Response, StatusCode> {
        let auth_header = request
            .headers()
            .get("authorization")
            .and_then(|v| v.to_str().ok());

        if let Some(auth) = auth_header {
            if auth.starts_with("Bearer ") {
                let token = &auth[7..];
                // Token validation would happen here
                // For now, pass through
                return Ok(next.run(request).await);
            }
        }

        // Allow unauthenticated routes to pass through
        Ok(next.run(request).await)
    }
}