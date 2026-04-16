use config::{Config, ConfigError, File};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub database_url: String,
    pub redis_url: String,
    pub app_port: u16,
    pub jwt_secret: String,
    pub jwt_expiry_hours: i64,
    pub jwt_refresh_expiry_days: i64,
    pub minio_endpoint: String,
    pub minio_access_key: String,
    pub minio_secret_key: String,
    pub cors_origins: Vec<String>,
    pub app_env: String,
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let config = Config::builder()
            .add_source(File::with_name("config").required(false))
            .add_source(File::with_name(".env").required(false))
            .add_source(config::Environment::with_prefix("APP").separator("__"))
            .build()?;

        config.try_deserialize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_config_deserialize_all_fields() {
        let json = r#"{
            "database_url": "postgres://user:pass@localhost:5432/testdb",
            "redis_url": "redis://localhost:6379",
            "app_port": 8080,
            "jwt_secret": "test-secret-key",
            "jwt_expiry_hours": 1,
            "jwt_refresh_expiry_days": 7,
            "minio_endpoint": "localhost:9000",
            "minio_access_key": "minioadmin",
            "minio_secret_key": "minioadmin",
            "cors_origins": ["http://localhost:3000"],
            "app_env": "test"
        }"#;

        let config: AppConfig = serde_json::from_str(json).expect("should deserialize");
        assert_eq!(config.database_url, "postgres://user:pass@localhost:5432/testdb");
        assert_eq!(config.redis_url, "redis://localhost:6379");
        assert_eq!(config.app_port, 8080);
        assert_eq!(config.jwt_secret, "test-secret-key");
        assert_eq!(config.jwt_expiry_hours, 1);
        assert_eq!(config.jwt_refresh_expiry_days, 7);
        assert_eq!(config.minio_endpoint, "localhost:9000");
        assert_eq!(config.app_env, "test");
        assert_eq!(config.cors_origins.len(), 1);
    }

    #[test]
    fn test_app_config_jwt_expiry_defaults() {
        let json = r#"{
            "database_url": "postgres://localhost/test",
            "redis_url": "redis://localhost",
            "app_port": 8080,
            "jwt_secret": "secret",
            "jwt_expiry_hours": 24,
            "jwt_refresh_expiry_days": 30,
            "minio_endpoint": "localhost:9000",
            "minio_access_key": "key",
            "minio_secret_key": "key",
            "cors_origins": [],
            "app_env": "development"
        }"#;

        let config: AppConfig = serde_json::from_str(json).expect("should deserialize");
        assert_eq!(config.jwt_expiry_hours, 24);
        assert_eq!(config.jwt_refresh_expiry_days, 30);
    }

    #[test]
    fn test_app_config_empty_cors_origins() {
        let json = r#"{
            "database_url": "postgres://localhost/test",
            "redis_url": "redis://localhost",
            "app_port": 8080,
            "jwt_secret": "secret",
            "jwt_expiry_hours": 1,
            "jwt_refresh_expiry_days": 7,
            "minio_endpoint": "localhost:9000",
            "minio_access_key": "key",
            "minio_secret_key": "key",
            "cors_origins": [],
            "app_env": "test"
        }"#;

        let config: AppConfig = serde_json::from_str(json).expect("should deserialize");
        assert!(config.cors_origins.is_empty());
    }

    #[test]
    fn test_app_config_multiple_cors_origins() {
        let json = r#"{
            "database_url": "postgres://localhost/test",
            "redis_url": "redis://localhost",
            "app_port": 8080,
            "jwt_secret": "secret",
            "jwt_expiry_hours": 1,
            "jwt_refresh_expiry_days": 7,
            "minio_endpoint": "localhost:9000",
            "minio_access_key": "key",
            "minio_secret_key": "key",
            "cors_origins": ["http://localhost:3000", "https://app.garage360.com", "https://staging.garage360.com"],
            "app_env": "production"
        }"#;

        let config: AppConfig = serde_json::from_str(json).expect("should deserialize");
        assert_eq!(config.cors_origins.len(), 3);
        assert_eq!(config.cors_origins[0], "http://localhost:3000");
        assert_eq!(config.app_env, "production");
    }
}
