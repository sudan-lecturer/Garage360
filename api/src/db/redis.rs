use redis::{aio::ConnectionManager, Client};

pub async fn create_client(redis_url: &str) -> anyhow::Result<ConnectionManager> {
    let client = Client::open(redis_url)
        .map_err(|e| anyhow::anyhow!("Failed to create Redis client: {}", e))?;
    
    ConnectionManager::new(client)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect to Redis: {}", e))
}
