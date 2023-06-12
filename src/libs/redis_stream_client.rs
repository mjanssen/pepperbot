use redis::{Client, Commands, Connection, RedisError, RedisResult};

use thiserror::Error;

use crate::structs::message::{Message, LIST_NAME};

use super::redis::{Config, Database};

#[derive(Debug, Error)]
pub enum RedisStreamError {
    #[error(transparent)]
    FailedCreateStream(#[from] RedisError),
}

pub struct RedisStreamClient {
    pub client: Client,
}

impl RedisStreamClient {
    pub fn create_generic_config(&self) -> Result<(), RedisStreamError> {
        let mut con: Connection = self.get_connection()?;

        // Make the current connection connect to the messages database
        let _: Result<(), redis::RedisError> = redis::cmd("SELECT")
            .arg(Database::CONFIG as u8)
            .query(&mut con);

        let _: Result<u8, redis::RedisError> = redis::cmd("SETNX")
            .arg(Config::OperationalKey.value())
            .arg(1)
            .query::<u8>(&mut con);

        let _: Result<u8, redis::RedisError> = redis::cmd("SETNX")
            .arg(Config::MessagesSentKey.value())
            .arg(0)
            .query::<u8>(&mut con);

        let _: Result<u8, redis::RedisError> = redis::cmd("SETNX")
            .arg(Config::DealsSentKey.value())
            .arg(0)
            .query::<u8>(&mut con);

        Ok(())
    }

    pub fn get_connection(&self) -> RedisResult<Connection> {
        self.client.get_connection()
    }

    pub fn read(&self, con: &mut Connection) -> Option<Message> {
        // Make sure we're using the message database
        let _: Result<(), redis::RedisError> =
            redis::cmd("SELECT").arg(Database::MESSAGE as u8).query(con);

        let read: RedisResult<(String, String)> = con.blpop(LIST_NAME, 0);
        if let Ok((_list, list_message)) = read {
            if let Ok(message) = serde_json::from_str::<Message>(&list_message) {
                return Some(message);
            }
        }

        None
    }
}
