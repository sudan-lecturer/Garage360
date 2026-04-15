use lru::LruCache;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Duration;

pub struct TenantPoolRegistry {
    pools: Arc<RwLock<LruCache<String, PgPool>>>,
    max_pools: usize,
}

impl TenantPoolRegistry {
    pub fn new(max_pools: usize) -> Self {
        Self {
            pools: Arc::new(RwLock::new(LruCache::new(max_pools))),
            max_pools,
        }
    }

    pub async fn get_pool(&self, tenant_id: &str, database_url: &str) -> anyhow::Result<PgPool> {
        {
            let pools = self.pools.read().await;
            if let Some(pool) = pools.get(tenant_id) {
                return Ok(pool.clone());
            }
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

        {
            let mut pools = self.pools.write().await;
            if pools.len() >= self.max_pools {
                pools.pop_lru();
            }
            pools.put(tenant_id.to_string(), pool.clone());
        }

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
