use std::env;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RedisError {
    #[error(transparent)]
    ReqwestError(#[from] redis::RedisError),

    #[error("Redis url not set in ENV")]
    RedisUrlMissing,
}

#[derive(Clone)]
pub struct RedisService {
    pub client: redis::Client,
}

pub fn get_redis_service() -> Result<RedisService, RedisError> {
    if let Ok(redis_domain) = env::var("REDIS_URL") {
        let redis_service = RedisService {
            client: redis::Client::open(redis_domain)?,
        };

        return Ok(redis_service);
    }

    Err(RedisError::RedisUrlMissing)
}
