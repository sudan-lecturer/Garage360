use axum::{
    async_trait,
    extract::FromRequestParts,
    http::request::Parts,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

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
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let app_state = state;

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

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SECRET: &str = "test-jwt-secret-key-for-testing-12345";

    fn create_test_jwt_service() -> JwtService {
        JwtService::new(TEST_SECRET)
    }

    #[test]
    fn test_jwt_service_new_creates_keys_from_secret() {
        let service = JwtService::new("my-secret-key");
        let token = service
            .create_access_token("user-123", "tenant-456", "ADMIN", 1)
            .expect("should create token");
        let claims = service.decode(&token).expect("should decode");
        assert_eq!(claims.sub, "user-123");
    }

    #[test]
    fn test_create_access_token_produces_valid_jwt() {
        let service = create_test_jwt_service();
        let token = service
            .create_access_token("user-123", "tenant-456", "ADMIN", 1)
            .expect("should create token");

        assert!(!token.is_empty());
        assert!(token.contains('.'));
        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3);
    }

    #[test]
    fn test_create_refresh_token_produces_valid_jwt() {
        let service = create_test_jwt_service();
        let token = service
            .create_refresh_token("user-123", "tenant-456", "MECHANIC", 7)
            .expect("should create token");

        assert!(!token.is_empty());
        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3);
    }

    #[test]
    fn test_access_token_has_kid_default() {
        let service = create_test_jwt_service();
        let token = service
            .create_access_token("user-123", "tenant-456", "ADMIN", 1)
            .expect("should create token");

        let claims = service.decode(&token).expect("should decode");
        assert_eq!(claims.kid, "default");
    }

    #[test]
    fn test_refresh_token_has_kid_refresh() {
        let service = create_test_jwt_service();
        let token = service
            .create_refresh_token("user-123", "tenant-456", "ADMIN", 7)
            .expect("should create token");

        let claims = service.decode(&token).expect("should decode");
        assert_eq!(claims.kid, "refresh");
    }

    #[test]
    fn test_decode_returns_correct_claims() {
        let service = create_test_jwt_service();
        let token = service
            .create_access_token("user-abc", "tenant-xyz", "MANAGER", 2)
            .expect("should create token");

        let claims = service.decode(&token).expect("should decode");

        assert_eq!(claims.sub, "user-abc");
        assert_eq!(claims.tenant_id, "tenant-xyz");
        assert_eq!(claims.role, "MANAGER");
        assert!(claims.exp > claims.iat);
    }

    #[test]
    fn test_encode_and_decode_round_trip() {
        let service = create_test_jwt_service();
        let original_claims = Claims {
            sub: "round-trip-user".to_string(),
            tenant_id: "round-trip-tenant".to_string(),
            role: "CASHIER".to_string(),
            exp: chrono::Utc::now().timestamp() + 3600,
            iat: chrono::Utc::now().timestamp(),
            kid: "test".to_string(),
        };

        let token = service.encode(&original_claims).expect("should encode");
        let decoded = service.decode(&token).expect("should decode");

        assert_eq!(decoded.sub, original_claims.sub);
        assert_eq!(decoded.tenant_id, original_claims.tenant_id);
        assert_eq!(decoded.role, original_claims.role);
    }

    #[test]
    fn test_decode_invalid_token_returns_error() {
        let service = create_test_jwt_service();
        let result = service.decode("invalid.token.here");
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_token_from_different_secret_returns_error() {
        let service_a = JwtService::new("secret-a");
        let service_b = JwtService::new("secret-b");

        let token = service_a
            .create_access_token("user", "tenant", "ADMIN", 1)
            .expect("should create token");

        let result = service_b.decode(&token);
        assert!(result.is_err());
    }

    #[test]
    fn test_access_token_expiry_is_calculated_correctly() {
        let service = create_test_jwt_service();
        let expiry_hours: i64 = 24;
        let token = service
            .create_access_token("user", "tenant", "ADMIN", expiry_hours)
            .expect("should create token");

        let claims = service.decode(&token).expect("should decode");
        let expected_exp = claims.iat + (expiry_hours * 3600);
        assert_eq!(claims.exp, expected_exp);
    }

    #[test]
    fn test_refresh_token_expiry_is_calculated_correctly() {
        let service = create_test_jwt_service();
        let expiry_days: i64 = 30;
        let token = service
            .create_refresh_token("user", "tenant", "ADMIN", expiry_days)
            .expect("should create token");

        let claims = service.decode(&token).expect("should decode");
        let expected_exp = claims.iat + (expiry_days * 24 * 3600);
        assert_eq!(claims.exp, expected_exp);
    }

    #[test]
    fn test_auth_user_from_claims() {
        let claims = Claims {
            sub: "user-id-123".to_string(),
            tenant_id: "tenant-id-456".to_string(),
            role: "MECHANIC".to_string(),
            exp: 9999999999,
            iat: 1000000000,
            kid: "default".to_string(),
        };

        let auth_user = AuthUser::from_claims(&claims);

        assert_eq!(auth_user.user_id, "user-id-123");
        assert_eq!(auth_user.tenant_id, "tenant-id-456");
        assert_eq!(auth_user.role, "MECHANIC");
    }

    #[test]
    fn test_auth_user_clone_preserves_data() {
        let auth_user = AuthUser {
            user_id: "clone-test".to_string(),
            tenant_id: "tenant-clone".to_string(),
            role: "OWNER".to_string(),
        };

        let cloned = auth_user.clone();
        assert_eq!(cloned.user_id, auth_user.user_id);
        assert_eq!(cloned.tenant_id, auth_user.tenant_id);
        assert_eq!(cloned.role, auth_user.role);
    }

    #[test]
    fn test_claims_serialize_and_deserialize() {
        let claims = Claims {
            sub: "serialize-test".to_string(),
            tenant_id: "tenant-ser".to_string(),
            role: "HR_OFFICER".to_string(),
            exp: 9999999999,
            iat: 1000000000,
            kid: "default".to_string(),
        };

        let json = serde_json::to_string(&claims).expect("should serialize");
        let deserialized: Claims =
            serde_json::from_str(&json).expect("should deserialize");

        assert_eq!(deserialized.sub, claims.sub);
        assert_eq!(deserialized.tenant_id, claims.tenant_id);
        assert_eq!(deserialized.role, claims.role);
    }
}
