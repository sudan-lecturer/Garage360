use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
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
    #[serde(default)]
    pub cors_origins: Vec<String>,
    pub app_env: String,
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        Config::builder()
            .add_source(File::with_name("config").required(false))
            .add_source(File::with_name(".env").required(false))
            .add_source(Environment::default().try_parsing(true).separator("__"))
            .set_default("app_port", 8080)?
            .set_default("jwt_expiry_hours", 1)?
            .set_default("jwt_refresh_expiry_days", 7)?
            .set_default("app_env", "development")?
            .set_default("cors_origins", Vec::<String>::new())?
            .build()?
            .try_deserialize()
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
        assert_eq!(config.cors_origins, vec!["http://localhost:3000"]);
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
}
