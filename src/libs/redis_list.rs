use serde::{Deserialize, Serialize};

use redis::{
    Client, Connection, RedisError, RedisResult,
};

use uuid::Uuid;

use super::redis::{Database, Config};

const LIST_NAME: &str = "messages_list";

#[derive(Serialize, Deserialize)]
pub struct ListEntry {
    pub message_id: String,
    pub link: String,
    pub title: String,
    pub category: String,
}

pub struct RedisList {
    pub client: Client,
}

impl RedisList {
    // Create generic config for the application. SETNX is used to keep existing config
    pub fn create_generic_config(&self) -> Result<(), RedisError> {
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

    pub fn consumer_name(&self) -> String {
        Uuid::new_v4().to_string()
    }

    pub fn add(
        &self,
        list_entry: ListEntry,
        con: &mut Connection,
    ) -> Result<String, RedisError> {
        // Serialize it to a JSON string.
        let j = serde_json::to_string(&list_entry);
        let jw = j.unwrap();

        let _: Result<(), redis::RedisError> = redis::cmd("LPUSH")
            .arg(LIST_NAME)
            .arg(&jw)
            .query(con);

        Ok(jw)
    }

    pub fn read(
        &self,
        con: &mut Connection,
    ) -> Result<ListEntry, RedisError> {
        let result: Result<(), redis::RedisError> = redis::cmd("BLPOP")
            .arg(LIST_NAME)
            .arg(0)
            .query(con);

        let d: ListEntry = serde_json::from_str(&result).unwrap();

        Ok(d)
    }
}
