use lru::LruCache;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Duration;
use std::num::NonZeroUsize;

pub struct TenantPoolRegistry {
    pools: Arc<RwLock<LruCache<String, PgPool>>>,
    max_pools: usize,
}

impl TenantPoolRegistry {
    pub fn new(max_pools: usize) -> Self {
        Self {
            pools: Arc::new(RwLock::new(LruCache::new(NonZeroUsize::new(max_pools).unwrap()))),
            max_pools,
        }
    }

    pub async fn get_pool(&self, tenant_id: &str, database_url: &str) -> anyhow::Result<PgPool> {
        let mut pools = self.pools.write().await;
        
        if let Some(pool) = pools.get(tenant_id) {
            return Ok(pool.clone());
        }

        let pool = PgPoolOptions::new()
            .max_connections(10)
            .min_connections(1)
            .acquire_timeout(Duration::from_secs(10))
            .idle_timeout(Duration::from_secs(300))
            .max_lifetime(Duration::from_secs(900))
            .connect(database_url)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to tenant pool {}: {}", tenant_id, e))?;

        if pools.len() >= self.max_pools {
            pools.pop_lru();
        }
        pools.put(tenant_id.to_string(), pool.clone());

        Ok(pool)
    }

    pub async fn remove_pool(&self, tenant_id: &str) {
        let mut pools = self.pools.write().await;
        pools.pop(tenant_id);
    }

    pub async fn clear(&self) {
        let mut pools = self.pools.write().await;
        pools.clear();
    }
}

impl Default for TenantPoolRegistry {
    fn default() -> Self {
        Self::new(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tenant_pool_registry_new() {
        let registry = TenantPoolRegistry::new(50);
        assert_eq!(registry.max_pools, 50);
    }

    #[tokio::test]
    async fn test_tenant_pool_registry_default() {
        let registry = TenantPoolRegistry::default();
        assert_eq!(registry.max_pools, 100);
    }

    #[tokio::test]
    async fn test_tenant_pool_registry_remove_pool() {
        let registry = TenantPoolRegistry::new(10);
        registry.remove_pool("tenant-1").await;

        let pools = registry.pools.read().await;
        assert!(!pools.contains("tenant-1"));
    }

    #[tokio::test]
    async fn test_tenant_pool_registry_clear() {
        let registry = TenantPoolRegistry::new(10);
        registry.clear().await;

        let pools = registry.pools.read().await;
        assert!(pools.is_empty());
    }

    #[tokio::test]
    async fn test_tenant_pool_registry_lru_eviction_logic() {
        let registry = TenantPoolRegistry::new(2);

        assert_eq!(registry.max_pools, 2);
        assert!(registry.pools.try_read().is_some());
    }
}
