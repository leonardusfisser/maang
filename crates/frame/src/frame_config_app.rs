use crate::frame_error_app::AppError;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub security: SecurityConfig,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]  // Added Copy
pub struct ServerConfig {
    pub bind_address: SocketAddr,
    pub worker_threads: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub pool_size: u32,
    pub connection_timeout_secs: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]  // Added Copy
pub struct SecurityConfig {
    pub session_ttl_secs: u64,
    pub csrf_token_ttl_secs: u64,
    pub rate_limit_requests: u32,
    pub rate_limit_window_secs: u64,
}

impl AppConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, AppError> {
        let bind_address = std::env::var("BIND_ADDRESS")
            .unwrap_or_else(|_| "127.0.0.1:8080".to_string())
            .parse()
            .map_err(|e| AppError::config(format!("Invalid bind address: {e}")))?;

        let worker_threads = std::env::var("WORKER_THREADS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(num_cpus::get);

        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| AppError::config("DATABASE_URL not set"))?;

        let redis_url = std::env::var("REDIS_URL")
            .map_err(|_| AppError::config("REDIS_URL not set"))?;

        Ok(Self {
            server: ServerConfig {
                bind_address,
                worker_threads,
            },
            database: DatabaseConfig {
                url: database_url,
                max_connections: 20,
                min_connections: 5,
                connection_timeout_secs: 30,
            },
            redis: RedisConfig {
                url: redis_url,
                pool_size: 10,
                connection_timeout_secs: 5,
            },
            security: SecurityConfig {
                session_ttl_secs: 3600,
                csrf_token_ttl_secs: 3600,
                rate_limit_requests: 100,
                rate_limit_window_secs: 60,
            },
        })
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), AppError> {
        if self.database.max_connections < self.database.min_connections {
            return Err(AppError::config(
                "max_connections must be >= min_connections",
            ));
        }

        if self.redis.pool_size == 0 {
            return Err(AppError::config("redis pool_size must be > 0"));
        }

        if self.security.session_ttl_secs == 0 {
            return Err(AppError::config("session_ttl_secs must be > 0"));
        }

        Ok(())
    }
}