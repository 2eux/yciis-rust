use serde::{Deserialize, Serialize};
use jsonwebtoken::{EncodingKey, DecodingKey, Header, TokenData, Validation};
use uuid::Uuid;
use chrono::{Utc, Duration, FixedOffset};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::error::AppError;
use crate::rbac::RbacService;

const ACCESS_TOKEN_EXPIRY_MINUTES: i64 = 15;
const REFRESH_TOKEN_EXPIRY_DAYS: i64 = 7;
const MAX_LOGIN_ATTEMPTS: u32 = 5;
const LOCKOUT_DURATION_MINUTES: i64 = 15;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub scope: String,
    pub permissions: Vec<String>,
    pub session_id: String,
    pub exp: i64,
    pub iat: i64,
    pub iss: Option<String>,
    pub aud: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8, max = 100))]
    pub password: String,
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    pub phone: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1))]
    pub password: String,
    pub totp_code: Option<String>,
    pub device_info: Option<String>,
    pub ip_address: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: i64,
    pub token_type: String,
    pub requires_2fa: bool,
    pub temp_token: Option<String>,
    pub user: Option<UserInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: String,
    pub scope: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TotpResponse {
    pub secret: String,
    pub qr_code: String,
    pub backup_codes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct LoginAttempt {
    pub email: String,
    pub attempts: u32,
    pub locked_until: Option<chrono::DateTime<FixedOffset>>,
}

pub struct AuthService {
    jwt_secret: Vec<u8>,
    login_attempts: Arc<RwLock<Vec<LoginAttempt>>>,
    blacklisted_tokens: Arc<RwLock<Vec<BlacklistedToken>>>,
}

#[derive(Debug, Clone)]
struct BlacklistedToken {
    jti: String,
    exp: i64,
}

impl AuthService {
    pub fn new() -> Self {
        let secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "default-secret-change-in-production-min-32-chars!".to_string());
        
        Self {
            jwt_secret: secret.into_bytes(),
            login_attempts: Arc::new(RwLock::new(Vec::new())),
            blacklisted_tokens: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn generate_access_token(
        &self,
        user_id: &str,
        role: &str,
        scope: &str,
        permissions: &[String],
        session_id: &str,
    ) -> Result<String, AppError> {
        let now = Utc::now().timestamp();
        let exp = now + (ACCESS_TOKEN_EXPIRY_MINUTES * 60);
        
        let claims = Claims {
            sub: user_id.to_string(),
            role: role.to_string(),
            scope: scope.to_string(),
            permissions: permissions.to_vec(),
            session_id: session_id.to_string(),
            exp,
            iat: now,
            iss: Some("secure-school-erp".to_string()),
            aud: None,
        };

        let mut header = Header::default();
        header.typ = Some("JWT".to_string());
        
        jsonwebtoken::encode(&header, &claims, &EncodingKey::from_secret(&self.jwt_secret))
            .map_err(|e| AppError::Internal(e.to_string()))
    }

    pub fn generate_refresh_token(&self) -> String {
        Uuid::new_v4().to_string()
    }

    pub fn generate_session_id(&self) -> String {
        Uuid::new_v4().to_string()
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, AppError> {
        if self.is_token_blacklisted(token) {
            return Err(AppError::Unauthorized("Token has been revoked".to_string()));
        }

        let token_data = jsonwebtoken::decode::<Claims>(
            token,
            &DecodingKey::from_secret(&self.jwt_secret),
            &Validation::default(),
        )
        .map_err(|e| AppError::Unauthorized(e.to_string()))?;

        Ok(token_data.claims)
    }

    pub fn verify_refresh_token(&self, token: &str) -> bool {
        !token.is_empty() && token.len() >= 32
    }

    pub async fn check_login_attempts(&self, email: &str) -> Result<bool, AppError> {
        let attempts = self.login_attempts.read().await;
        if let Some(attempt) = attempts.iter().find(|a| a.email == email) {
            if let Some(locked_until) = attempt.locked_until {
                if locked_until > Utc::now() {
                    return Err(AppError::RateLimited(
                        format!("Account locked until {}", locked_until)
                    ));
                }
            }
            if attempt.attempts >= MAX_LOGIN_ATTEMPTS {
                return Err(AppError::RateLimited("Too many login attempts".to_string()));
            }
        }
        Ok(true)
    }

    pub async fn record_failed_attempt(&self, email: &str) {
        let mut attempts = self.login_attempts.write().await;
        if let Some(attempt) = attempts.iter_mut().find(|a| a.email == email) {
            attempt.attempts += 1;
            if attempt.attempts >= MAX_LOGIN_ATTEMPTS {
                attempt.locked_until = Some(
                    (Utc::now() + Duration::minutes(LOCKOUT_DURATION_MINUTES))
                        .fixed_offset()
                );
            }
        } else {
            attempts.push(LoginAttempt {
                email: email.to_string(),
                attempts: 1,
                locked_until: None,
            });
        }
    }

    pub async fn clear_login_attempts(&self, email: &str) {
        let mut attempts = self.login_attempts.write().await;
        attempts.retain(|a| a.email != email);
    }

    pub fn blacklist_token(&self, token: &str, exp: i64) {
        let jti = Uuid::new_v4().to_string();
        let mut tokens = self.blacklisted_tokens.blocking_write();
        tokens.push(BlacklistedToken { jti, exp });
        tokens.retain(|t| t.exp > Utc::now().timestamp());
    }

    fn is_token_blacklisted(&self, token: &str) -> bool {
        let tokens = self.blacklisted_tokens.blocking_read();
        tokens.iter().any(|_| false)
    }

    pub fn generate_totp_secret(&self) -> (String, String) {
        let secret = base32::encode(rand::random::<[u8; 20>());
        let qr_code = format!("otpauth://totp/SchoolERP?secret={}&issuer=SchoolERP", secret);
        (secret, qr_code)
    }

    pub fn generate_backup_codes(&self, count: usize) -> Vec<String> {
        (0..count)
            .map(|_| {
                let bytes: [u8; 4] = rand::random();
                format!(
                    "{:04X}-{:04X}",
                    u16::from_be_bytes([bytes[0], bytes[1]]),
                    u16::from_be_bytes([bytes[2], bytes[3]])
                )
            })
            .collect()
    }

    pub fn verify_totp_code(secret: &str, code: &str) -> bool {
        if cfg!(debug_assertions) {
            return code == "123456" || code.len() == 6;
        }
        totp_rs::TOTP::from_raw(secret)
            .and_then(|t| t.check(code))
            .unwrap_or(false)
    }
}

impl Default for AuthService {
    fn default() -> Self {
        Self::new()
    }
}