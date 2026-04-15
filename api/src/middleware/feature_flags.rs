use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::AppState;

#[derive(Debug, Clone)]
pub struct FeatureFlags {
    flags: HashMap<String, bool>,
}

impl FeatureFlags {
    pub fn is_enabled(&self, key: &str) -> bool {
        self.flags.get(key).copied().unwrap_or(false)
    }

    pub fn get_all(&self) -> &HashMap<String, bool> {
        &self.flags
    }
}

pub async fn load_feature_flags(
    db: &sqlx::PgPool,
    tenant_id: &str,
) -> Result<FeatureFlags, sqlx::Error> {
    let mut flags = HashMap::new();

    let defaults = sqlx::query_scalar::<_, (String, bool)>(
        "SELECT key, default_enabled FROM feature_flags"
    )
    .fetch_all(db)
    .await?;

    for (key, default) in defaults {
        flags.insert(key, default);
    }

    let overrides = sqlx::query_scalar::<_, (String, bool)>(
        r#"
        SELECT ff.key, tfo.is_enabled
        FROM tenant_feature_flag_overrides tfo
        JOIN feature_flags ff ON ff.id = tfo.feature_flag_id
        WHERE tfo.tenant_id = $1
        "#
    )
    .bind(tenant_id)
    .fetch_all(db)
    .await?;

    for (key, enabled) in overrides {
        flags.insert(key, enabled);
    }

    Ok(FeatureFlags { flags })
}

#[derive(Debug, Clone)]
pub struct CachedFeatureFlags {
    cache: Arc<RwLock<HashMap<String, FeatureFlags>>>,
    ttl_secs: u64,
}

impl Default for CachedFeatureFlags {
    fn default() -> Self {
        Self::new(300)
    }
}

impl CachedFeatureFlags {
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl_secs,
        }
    }

    pub async fn get_flags(
        &self,
        db: &sqlx::PgPool,
        tenant_id: &str,
    ) -> Result<FeatureFlags, sqlx::Error> {
        {
            let cache = self.cache.read().await;
            if let Some(flags) = cache.get(tenant_id) {
                return Ok(flags.clone());
            }
        }

        let flags = load_feature_flags(db, tenant_id).await?;

        {
            let mut cache = self.cache.write().await;
            cache.insert(tenant_id.to_string(), flags.clone());
        }

        Ok(flags)
    }

    pub async fn invalidate(&self, tenant_id: &str) {
        let mut cache = self.cache.write().await;
        cache.remove(tenant_id);
    }

    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }
}
