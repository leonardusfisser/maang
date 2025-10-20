use redis::AsyncCommands;
use core_errors::SecurityError;

#[derive(Debug)]
pub struct RateLimiter {
    client: redis::Client,
}

impl RateLimiter {
    pub fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        Ok(Self {
            client: redis::Client::open(redis_url)?,
        })
    }

    pub async fn check_login(&self, ip: &str) -> Result<(), SecurityError> {
        let key = format!("ratelimit:login:{}", ip);
        let mut conn = self.client.get_multiplexed_async_connection().await
            .map_err(|_| SecurityError::RateLimited)?;

        let count: i32 = conn.get(&key).await.unwrap_or(0);

        if count >= 5 {
            return Err(SecurityError::RateLimited);
        }

        let _: () = conn.incr(&key, 1).await
            .map_err(|_| SecurityError::RateLimited)?;
        let _: () = conn.expire(&key, 300).await
            .map_err(|_| SecurityError::RateLimited)?;

        Ok(())
    }

    pub async fn reset_login(&self, ip: &str) -> Result<(), SecurityError> {
        let key = format!("ratelimit:login:{}", ip);
        let mut conn = self.client.get_multiplexed_async_connection().await
            .map_err(|_| SecurityError::RateLimited)?;

        let _: () = conn.del(&key).await
            .map_err(|_| SecurityError::RateLimited)?;
        Ok(())
    }
}