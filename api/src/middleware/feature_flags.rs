use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

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

    let defaults = sqlx::query_as::<_, (String, bool)>(
        "SELECT key, default_enabled FROM feature_flags"
    )
    .fetch_all(db)
    .await?;

    for (key, default) in defaults {
        flags.insert(key, default);
    }

    let overrides = sqlx::query_as::<_, (String, bool)>(
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
    _ttl_secs: u64,
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
            _ttl_secs: ttl_secs,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_flags_is_enabled_returns_true_for_enabled_flag() {
        let mut flags = HashMap::new();
        flags.insert("module.dvi".to_string(), true);
        flags.insert("module.hr".to_string(), false);

        let feature_flags = FeatureFlags { flags };

        assert!(feature_flags.is_enabled("module.dvi"));
        assert!(!feature_flags.is_enabled("module.hr"));
    }

    #[test]
    fn test_feature_flags_is_enabled_defaults_to_false_for_unknown() {
        let flags = HashMap::new();
        let feature_flags = FeatureFlags { flags };

        assert!(!feature_flags.is_enabled("module.dvi"));
        assert!(!feature_flags.is_enabled("nonexistent.flag"));
    }

    #[test]
    fn test_feature_flags_get_all_returns_hashmap() {
        let mut flags = HashMap::new();
        flags.insert("jobs.intake".to_string(), true);
        flags.insert("jobs.bay_management".to_string(), false);

        let feature_flags = FeatureFlags { flags };
        let all = feature_flags.get_all();

        assert_eq!(all.len(), 2);
        assert_eq!(all.get("jobs.intake"), Some(&true));
        assert_eq!(all.get("jobs.bay_management"), Some(&false));
    }

    #[tokio::test]
    async fn test_cached_feature_flags_new() {
        let cache = CachedFeatureFlags::new(600);
        assert_eq!(cache._ttl_secs, 600);
    }

    #[tokio::test]
    async fn test_cached_feature_flags_default_ttl() {
        let cache = CachedFeatureFlags::default();
        assert_eq!(cache._ttl_secs, 300);
    }

    #[test]
    fn test_feature_flags_multiple_flags() {
        let mut flags = HashMap::new();
        flags.insert("module.dvi".to_string(), true);
        flags.insert("module.purchases".to_string(), true);
        flags.insert("module.hr".to_string(), false);
        flags.insert("module.assets".to_string(), false);
        flags.insert("module.customer_portal".to_string(), false);
        flags.insert("jobs.intake_inspection".to_string(), true);
        flags.insert("jobs.bay_management".to_string(), true);
        flags.insert("jobs.approval_workflow".to_string(), true);

        let feature_flags = FeatureFlags { flags };

        assert!(feature_flags.is_enabled("module.dvi"));
        assert!(feature_flags.is_enabled("module.purchases"));
        assert!(!feature_flags.is_enabled("module.hr"));
        assert!(!feature_flags.is_enabled("module.assets"));
        assert!(!feature_flags.is_enabled("module.customer_portal"));
        assert!(feature_flags.is_enabled("jobs.intake_inspection"));
        assert!(feature_flags.is_enabled("jobs.bay_management"));
        assert!(feature_flags.is_enabled("jobs.approval_workflow"));
    }
}
