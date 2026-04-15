use axum::{
    async_trait,
    extract::FromRequestParts,
    http::request::Parts,
    middleware::Next,
    response::{IntoResponse, Response},
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::errors::AppError;
use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub tenant_id: String,
    pub role: String,
    pub exp: i64,
    pub iat: i64,
    pub kid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub user_id: String,
    pub tenant_id: String,
    pub role: String,
}

impl AuthUser {
    pub fn from_claims(claims: &Claims) -> Self {
        Self {
            user_id: claims.sub.clone(),
            tenant_id: claims.tenant_id.clone(),
            role: claims.role.clone(),
        }
    }
}

#[async_trait]
impl<S: Send + Sync + 'static> FromRequestParts<S> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = state
            .downcast_ref::<Arc<AppState>>()
            .ok_or_else(|| AppError::Unauthorized("Missing app state".into()))?;

        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".into()))?;

        if !auth_header.starts_with("Bearer ") {
            return Err(AppError::Unauthorized("Invalid Authorization header format".into()));
        }

        let token = &auth_header[7..];
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(app_state.config.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| AppError::Unauthorized(format!("Invalid token: {}", e)))?;

        Ok(AuthUser::from_claims(&token_data.claims))
    }
}

pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtService {
    pub fn new(secret: &str) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
        }
    }

    pub fn encode(&self, claims: &Claims) -> Result<String, AppError> {
        encode(&Header::default(), claims, &self.encoding_key).map_err(AppError::from)
    }

    pub fn decode(&self, token: &str) -> Result<Claims, AppError> {
        let token_data = decode::<Claims>(
            token,
            &self.decoding_key,
            &Validation::default(),
        )
        .map_err(AppError::from)?;
        Ok(token_data.claims)
    }

    pub fn create_access_token(
        &self,
        user_id: &str,
        tenant_id: &str,
        role: &str,
        expiry_hours: i64,
    ) -> Result<String, AppError> {
        let now = chrono::Utc::now().timestamp();
        let claims = Claims {
            sub: user_id.to_string(),
            tenant_id: tenant_id.to_string(),
            role: role.to_string(),
            exp: now + (expiry_hours * 3600),
            iat: now,
            kid: "default".to_string(),
        };
        self.encode(&claims)
    }

    pub fn create_refresh_token(
        &self,
        user_id: &str,
        tenant_id: &str,
        role: &str,
        expiry_days: i64,
    ) -> Result<String, AppError> {
        let now = chrono::Utc::now().timestamp();
        let claims = Claims {
            sub: user_id.to_string(),
            tenant_id: tenant_id.to_string(),
            role: role.to_string(),
            exp: now + (expiry_days * 24 * 3600),
            iat: now,
            kid: "refresh".to_string(),
        };
        self.encode(&claims)
    }
}
