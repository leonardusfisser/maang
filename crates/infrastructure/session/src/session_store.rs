use redis::AsyncCommands;
use core_errors::SessionError;
use domain_sessions::Session;
use rand::{Rng, distr::Alphanumeric};

#[derive(Debug)]
pub struct RedisSessionStore {
    client: redis::Client,
}

impl RedisSessionStore {
    pub fn new(redis_url: &str) -> Result<Self, SessionError> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self { client })
    }

    pub async fn create(&self, token: &str, session: &Session, ttl: u64) -> Result<(), SessionError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let data = serde_json::to_string(session)
            .map_err(|_| SessionError::Invalid)?;

        conn.set_ex::<_, _, ()>(format!("session:{}", token), data, ttl).await?;
        Ok(())
    }

    pub async fn get(&self, token: &str) -> Result<Session, SessionError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let data: Option<String> = conn.get(format!("session:{}", token)).await?;

        match data {
            Some(json) => serde_json::from_str(&json)
                .map_err(|_| SessionError::Invalid),
            None => Err(SessionError::NotFound),
        }
    }

    pub async fn delete(&self, token: &str) -> Result<(), SessionError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        conn.del::<_, ()>(format!("session:{}", token)).await?;
        Ok(())
    }

    pub async fn refresh(&self, token: &str, ttl: u64) -> Result<(), SessionError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        conn.expire::<_, ()>(format!("session:{}", token), ttl as i64).await?;
        Ok(())
    }
}

impl RedisSessionStore {
    pub async fn rotate(&self, old_token: &str, ttl: u64) -> Result<String, SessionError> {
        let session = self.get(old_token).await?;

        let new_token: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        self.create(&new_token, &session, ttl).await?;

        // 30s overlap for concurrent requests
        self.refresh(old_token, 30).await?;

        Ok(new_token)
    }

    pub async fn validate_token(&self, token: &str) -> Result<(), SessionError> {
        if token.len() != 32 || !token.chars().all(|c| c.is_alphanumeric()) {
            return Err(SessionError::Invalid);
        }
        Ok(())
    }
}