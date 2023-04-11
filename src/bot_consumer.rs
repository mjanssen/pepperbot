pub mod libs;

use libs::redis::Database;
use libs::redis_stream_client::{RedisStreamClient, StreamEntry};
use libs::telegram::BotMessageService;
use log::info;
use redis::{streams::StreamId, ConnectionLike};
use std::env;
use thiserror::Error;

use teloxide::Bot;

use regex::Regex;

use crate::libs::version::print_version;
use crate::libs::redis::{get_config, get_subscribers, increase_config_value};

#[derive(Debug, Error)]
enum ConsumerError {
    #[error(transparent)]
    RedisError(#[from] redis::RedisError),
}

#[tokio::main]
async fn main() -> Result<(), ConsumerError> {
    env_logger::init();
    print_version();

    info!("Starting bot consumer service");

    if let Ok(redis_domain) = env::var("REDIS_URL") {
        match redis::Client::open(redis_domain.clone()) {
            Ok(redis_client) => {
                // Create a sanitize regex to clean the title
                let sanitize_regex = Regex::new(r"([^\w\s\\'\\’\\$\\€])").unwrap();

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

                // Make sure the generic application config is set
                match stream_client.create_generic_config() {
                    _ => (),
                }

                loop {
                    if redis_client.is_open() == false {
                        panic!("Redis connection dropped");
                    }

                    if let Ok(mut con) = redis_client.get_connection() {
                        // Make the current connection connect to the messages database
                        let _: Result<(), redis::RedisError> = redis::cmd("SELECT")
                            .arg(Database::MESSAGE as u8)
                            .query(&mut con);

                        let result = stream_client.read(&mut con, &consumer_name);

                        if let Ok(stream_key) = result {
                            for key in stream_key.keys {
                                if let Some(stream) = key.ids.first() {
                                    let stream_entry = process_stream_entry(stream);

                                    if stream_entry.message_id.eq("") {
                                        continue;
                                    }

                                    // Check if the bot has been disabled by the admin
                                    let is_operational: String = get_config(
                                        &mut con,
                                        libs::redis::Config::OperationalKey,
                                        Database::MESSAGE,
                                    )
                                    .unwrap_or("1".to_string());

                                    if is_operational.eq(&"0") {
                                        continue;
                                    }

                                    let res: i64 = redis::cmd("EXISTS")
                                        .arg(&stream_entry.message_id)
                                        .query(&mut con)?;

                                    // Only send if the message has not been send yet
                                    if res == 1 {
                                        continue;
                                    }

                                    info!("Sending message {:?}", &stream_entry);

                                    let _ = increase_config_value::<()>(
                                        &mut con,
                                        libs::redis::Config::DealsSentKey,
                                        Database::MESSAGE,
                                        1
                                    );

                                    let subscribers = get_subscribers(redis_client.clone()).await;

                                    if let Ok(subs) = subscribers {
                                        let mut messages_sent = 0;

                                        for (chat_id, categories) in subs {
                                            let _: Result<(), redis::RedisError> =
                                                redis::cmd("SET")
                                                    .arg(&stream_entry.message_id)
                                                    .arg(1)
                                                    .query(&mut con);

                                            // Set expiration for key - 2 days
                                            let _: Result<(), redis::RedisError> =
                                                redis::cmd("EXPIRE")
                                                    .arg(&stream_entry.message_id)
                                                    .arg(172800)
                                                    .query(&mut con);

                                            stream_client.acknowledge(&mut con, &stream.id)?;

                                            // If user did not subscribe for this category, bail
                                            if let Some(c) = categories {
                                                if c.contains(&stream_entry.category) == false {
                                                    continue;
                                                }
                                            }

                                            let sanitized_title = sanitize_regex
                                                .replace_all(stream_entry.title.as_str(), "\\$1");

                                            let _ = bot_service
                                                .send_message(
                                                    chat_id,
                                                    format!(
                                                        "[{}]({})",
                                                        sanitized_title, stream_entry.link
                                                    ),
                                                )
                                                .await;

                                            messages_sent += 1;
                                        }

                                        let _ = increase_config_value::<()>(
                                            &mut con,
                                            libs::redis::Config::MessagesSentKey,
                                            Database::MESSAGE,
                                            messages_sent,
                                        );
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
    let message_id: String = match stream_entry.get("message_id") {
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
    let category: String = match stream_entry.get("category") {
        Some(v) => v,
        _ => "".to_string(),
    };

    StreamEntry {
        message_id,
        title,
        link,
        category,
    }
}
