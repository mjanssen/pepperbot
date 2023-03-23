use redis::{
    streams::{StreamMaxlen, StreamReadOptions, StreamReadReply},
    Client, Commands, Connection, RedisError, RedisResult,
};
use thiserror::Error;
use uuid::Uuid;

use super::redis::MESSAGE_DATABASE;

const STREAM_KEY: &str = "messages_stream_v2";
const GROUP_NAME: &str = "messages_consumer_v2";

#[derive(Debug, Error)]
pub enum RedisStreamError {
    #[error(transparent)]
    FailedCreateStream(#[from] RedisError),
}

pub struct StreamEntry {
    pub message_id: String,
    pub link: String,
    pub title: String,
    pub category: String,
}

pub struct RedisStreamClient {
    pub client: Client,
}

impl RedisStreamClient {
    pub fn create_group_and_stream(&self) -> Result<(), RedisStreamError> {
        let mut con: Connection = self.get_connection()?;

        // Make the current connection connect to the messages database
        let _: Result<(), redis::RedisError> =
            redis::cmd("SELECT").arg(MESSAGE_DATABASE).query(&mut con);

        match con.xgroup_create_mkstream(STREAM_KEY, GROUP_NAME, "$") {
            Ok(val) => val,
            Err(e) => {
                return Err(RedisStreamError::FailedCreateStream(e));
            }
        };

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
        stream_entry: StreamEntry,
        con: &mut Connection,
    ) -> Result<String, RedisError> {
        con.xadd_maxlen::<&str, &str, &str, &String, String>(
            STREAM_KEY,
            StreamMaxlen::Approx(100),
            "*",
            &[
                ("message_id", &stream_entry.message_id),
                ("link", &stream_entry.link),
                ("title", &stream_entry.title),
                ("category", &stream_entry.category),
            ],
        )
    }

    pub fn read(
        &self,
        con: &mut Connection,
        consumer_name: &String,
    ) -> RedisResult<StreamReadReply> {
        let opts: StreamReadOptions = StreamReadOptions::default()
            .count(1)
            .group(&GROUP_NAME, &consumer_name);

        con.xread_options(&[STREAM_KEY], &[">"], &opts)
    }

    pub fn acknowledge(&self, con: &mut Connection, stream_id: &String) -> RedisResult<()> {
        con.xack(&STREAM_KEY, &GROUP_NAME, &[stream_id])
    }
}
