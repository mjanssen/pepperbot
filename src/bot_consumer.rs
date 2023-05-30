pub mod libs;
pub mod structs;

use libs::redis::Database;
use libs::redis_stream_client::RedisStreamClient;
use libs::telegram::BotMessageService;
use log::{error, info};
use redis::ConnectionLike;
use std::{env, println};
use thiserror::Error;

use teloxide::Bot;

use regex::Regex;

use crate::libs::redis::{get_config, get_subscribers, increase_config_value};
use crate::libs::version::print_version;

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

    let sanitize_regex = Regex::new(r"([^\w\s\\'\\’\\$\\€])").unwrap();

    if let Ok(redis_domain) = env::var("REDIS_URL") {
        match redis::Client::open(redis_domain.clone()) {
            Ok(redis_client) => {
                let stream_client = RedisStreamClient {
                    client: redis_client.clone(),
                };

                let bot_service = BotMessageService {
                    bot: Bot::from_env(),
                };

                loop {
                    if redis_client.is_open() == false {
                        panic!("Redis connection dropped");
                    }

                    if let Ok(mut con) = redis_client.get_connection() {
                        if let Some(message) = stream_client.read(&mut con) {
                            info!("{}", message.id);

                            // Make sure we're using the message database
                            let _: Result<(), redis::RedisError> = redis::cmd("SELECT")
                                .arg(Database::MESSAGE as u8)
                                .query(&mut con);

                            let res: i64 = redis::cmd("EXISTS").arg(&message.id).query(&mut con)?;

                            println!("{:?}", res);

                            // Only send if the message has not been send yet
                            if res == 1 {
                                continue;
                            }

                            // Store this message in Redis to make sure it doesn't get
                            // queued again
                            let _: Result<(), redis::RedisError> =
                                redis::cmd("SET").arg(&message.id).arg(1).query(&mut con);

                            // Set expiration for key - 2 days
                            let _: Result<(), redis::RedisError> = redis::cmd("EXPIRE")
                                .arg(&message.id)
                                .arg(172800)
                                .query(&mut con);

                            // Check if the bot has been disabled by the admin
                            let is_operational: String = get_config(
                                &mut con,
                                libs::redis::Config::OperationalKey,
                                Database::MESSAGE,
                            )
                            .unwrap_or("1".to_string());

                            // Only send messages and get subs when we're operational
                            if is_operational.eq(&"1") {
                                let _ = increase_config_value::<()>(
                                    &mut con,
                                    libs::redis::Config::DealsSentKey,
                                    Database::MESSAGE,
                                    1,
                                );

                                info!("Sending message {:?}", &message);

                                let subscribers = get_subscribers(redis_client.clone()).await;
                                if let Ok(subs) = subscribers {
                                    let mut messages_sent = 0;

                                    for (chat_id, categories) in subs {
                                        // If user did not subscribe for this category, bail
                                        if let Some(c) = categories {
                                            if c.contains(&message.payload.category) == false {
                                                continue;
                                            }
                                        }

                                        let sanitized_title = sanitize_regex
                                            .replace_all(message.payload.title.as_str(), "\\$1");

                                        let _ = bot_service
                                            .send_message(
                                                chat_id,
                                                format!(
                                                    "[{}]({})",
                                                    sanitized_title, message.payload.link
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
            Err(_) => error!("Connection with Redis failed"),
        }
    }

    return Ok(());
}
