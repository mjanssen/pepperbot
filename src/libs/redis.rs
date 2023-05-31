use std::{collections::HashMap, env};

use log::info;
use redis::{Client, Commands, Connection, FromRedisValue, ToRedisArgs};
use thiserror::Error;

use crate::structs::message::{Message, MessageError};

#[derive(Error, Debug)]
pub enum RedisError {
    #[error(transparent)]
    RedisError(#[from] redis::RedisError),

    #[error("Redis url not set in ENV")]
    RedisUrlMissing,

    #[error("No subscribers found")]
    NoSubscribers,
}

#[derive(Clone)]
pub struct RedisService {
    pub client: redis::Client,
}

pub enum Database {
    SUBSCRIBER = 0,
    MESSAGE = 1,
    CONFIG = 2,
}

pub enum Config {
    OperationalKey,
    MessagesSentKey,
    DealsSentKey,
}

impl Config {
    pub fn value(&self) -> &str {
        match *self {
            Config::OperationalKey => "is_operational",
            Config::MessagesSentKey => "messages_sent_count",
            Config::DealsSentKey => "deals_sent_count",
        }
    }
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

pub async fn get_subscriber_amount(con: &mut Connection) -> usize {
    let _: Result<(), redis::RedisError> = redis::cmd("SELECT")
        .arg(Database::SUBSCRIBER as u8)
        .query(con);

    let keys: Result<Vec<String>, redis::RedisError> = redis::cmd("KEYS").arg("*").query(con);

    if let Ok(chat_ids) = keys {
        return chat_ids.len();
    }

    0
}

pub async fn get_subscribers(
    redis_client: Client,
) -> Result<HashMap<String, Option<Vec<String>>>, RedisError> {
    if let Ok(mut con) = redis_client.get_connection() {
        let _: Result<(), redis::RedisError> = redis::cmd("SELECT")
            .arg(Database::SUBSCRIBER as u8)
            .query(&mut con);

        let keys: Result<Vec<String>, redis::RedisError> =
            redis::cmd("KEYS").arg("*").query(&mut con);

        if let Ok(chat_ids) = keys {
            let mut subscribers: HashMap<String, Option<Vec<String>>> = HashMap::new();
            for chat_id in &chat_ids {
                let categories_string: String = redis::cmd("GET")
                    .arg(chat_id)
                    .query(&mut con)
                    .unwrap_or("".to_string());

                let categories: Vec<String> = categories_string
                    .split(",")
                    .map(|s| s.to_string())
                    .collect();

                // Fix for initial sign-ups
                if categories.contains(&"1".to_string()) {
                    subscribers.insert(chat_id.clone(), None);
                } else {
                    subscribers.insert(chat_id.clone(), Some(categories));
                }
            }

            return Ok(subscribers);
        }
    }

    Err(RedisError::NoSubscribers)
}

pub fn set_config<T: ToRedisArgs>(
    mut con: &mut Connection,
    config_key: Config,
    value: T,
) -> Result<(), RedisError> {
    let operational_key: &str = config_key.value();

    let _: Result<(), redis::RedisError> = redis::cmd("SELECT")
        .arg(Database::CONFIG as u8)
        .query(&mut con);

    let _: Result<(), redis::RedisError> = redis::cmd("SET")
        .arg(operational_key)
        .arg(value)
        .query(&mut con);

    Ok(())
}

pub fn increase_config_value<T: FromRedisValue>(
    mut con: &mut Connection,
    config_key: Config,
    next_database: Database,
    amount: u8,
) -> Result<(), RedisError> {
    let key: &str = config_key.value();

    let _: Result<(), redis::RedisError> = redis::cmd("SELECT")
        .arg(Database::CONFIG as u8)
        .query(&mut con);

    let _: Result<(), redis::RedisError> =
        redis::cmd("INCRBY").arg(key).arg(amount).query(&mut con);

    let _: Result<(), redis::RedisError> = redis::cmd("SELECT")
        .arg(next_database as u8)
        .query(&mut con);

    Ok(())
}

pub fn get_config<T: FromRedisValue>(
    mut con: &mut Connection,
    config_key: Config,
    next_database: Database,
) -> Option<T> {
    let key: &str = config_key.value();

    let _: Result<(), redis::RedisError> = redis::cmd("SELECT")
        .arg(Database::CONFIG as u8)
        .query(&mut con);

    let result: Result<T, redis::RedisError> = redis::cmd("GET").arg(key).query(&mut con);

    let _: Result<(), redis::RedisError> = redis::cmd("SELECT")
        .arg(next_database as u8)
        .query(&mut con);

    if let Ok(r) = result {
        return Some(r);
    }

    None
}

pub fn publish_message(redis_url: String, message: Message) -> Result<(), MessageError> {
    match redis::Client::open(redis_url) {
        Ok(redis_client) => match redis_client.get_connection() {
            Ok(mut con) => {
                if let Ok(json) = serde_json::to_string(&message) {
                    let _: Result<(), redis::RedisError> = redis::cmd("SELECT")
                        .arg(Database::MESSAGE as u8)
                        .query(&mut con);

                    match con.rpush::<String, String, i32>(message.list.clone(), json) {
                        Ok(e) => {
                            info!("Added message to list {}: {:?}", message.list, e);
                            return Ok(());
                        }
                        Err(e) => return Err(MessageError::RedisError(e)),
                    }
                }

                Err(MessageError::ParseError)
            }
            Err(e) => Err(MessageError::RedisError(e)),
        },
        Err(e) => Err(MessageError::RedisError(e)),
    }
}
