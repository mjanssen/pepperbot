pub mod libs;

use libs::redis::MESSAGE_DATABASE;
use libs::redis_stream_client::{RedisStreamClient, StreamEntry};
use libs::telegram::BotMessageService;
use redis::{streams::StreamId, ConnectionLike};
use std::env;
use thiserror::Error;

use teloxide::Bot;

#[derive(Debug, Error)]
enum ConsumerError {
    #[error(transparent)]
    RedisError(#[from] redis::RedisError),
}

#[tokio::main]
async fn main() -> Result<(), ConsumerError> {
    if let Ok(redis_domain) = env::var("REDIS_URL") {
        match redis::Client::open(redis_domain.clone()) {
            Ok(redis_client) => {
                let stream_client = RedisStreamClient {
                    client: redis_client.clone(),
                };

                let consumer_name = stream_client.consumer_name();
                let bot_service = BotMessageService {
                    bot: Bot::from_env(),
                };

                // We don't care for existing group errors
                match stream_client.create_group_and_stream() {
                    _ => (),
                }

                loop {
                    if redis_client.is_open() == false {
                        panic!("Redis connection dropped");
                    }

                    if let Ok(mut con) = redis_client.get_connection() {
                        // Make the current connection connect to the messages database
                        let _: Result<(), redis::RedisError> =
                            redis::cmd("SELECT").arg(MESSAGE_DATABASE).query(&mut con);

                        let result = stream_client.read(&mut con, &consumer_name);

                        if let Ok(stream_key) = result {
                            for key in stream_key.keys {
                                if let Some(stream) = key.ids.first() {
                                    let stream_entry = process_stream_entry(stream);
                                    let subscribers =
                                        bot_service.get_subscribers(redis_client.clone()).await;

                                    if let Ok(chat_ids) = subscribers {
                                        for chat_id in chat_ids {
                                            let _ = bot_service
                                                .send_message(
                                                    chat_id,
                                                    format!(
                                                        "{}\n{}",
                                                        stream_entry.title, stream_entry.link
                                                    ),
                                                )
                                                .await;

                                            let _: Result<(), redis::RedisError> =
                                                redis::cmd("SET").arg(&id).arg(1).query(&mut con);

                                            // Set expiration for key - 2 days
                                            let _: Result<(), redis::RedisError> =
                                                redis::cmd("EXPIRE")
                                                    .arg(&stream_entry.id)
                                                    .arg(172800)
                                                    .query(&mut con);

                                            stream_client.acknowledge(&mut con, &stream.id)?;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(_) => panic!("Could not connect to redis"),
        }
    }

    Ok(())
}

fn process_stream_entry(stream_entry: &StreamId) -> StreamEntry {
    let id: String = match stream_entry.get("id") {
        Some(v) => v,
        _ => "".to_string(),
    };

    let title: String = match stream_entry.get("title") {
        Some(v) => v,
        _ => "".to_string(),
    };

    let link: String = match stream_entry.get("link") {
        Some(v) => v,
        _ => "".to_string(),
    };

    // Can be used later on
    let category: String = match stream_entry.get("link") {
        Some(v) => v,
        _ => "".to_string(),
    };

    StreamEntry {
        id,
        title,
        link,
        category,
    }
}