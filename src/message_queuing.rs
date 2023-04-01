pub mod libs;

use libs::redis_stream_client::RedisStreamError;
use libs::rss::RSSError;
use log::{error, info, warn};
use rss::Channel;
use std::env;
use std::thread::sleep;
use std::time::Duration;

use crate::libs::redis::MESSAGE_DATABASE;
use crate::libs::redis_stream_client::{RedisStreamClient, StreamEntry};
use crate::libs::rss::get_rss_data;
use thiserror::Error;

#[derive(Debug, Error)]
enum QueuingError {
    #[error(transparent)]
    RedisError(#[from] redis::RedisError),

    #[error(transparent)]
    RSSError(#[from] RSSError),

    #[error(transparent)]
    StreamError(#[from] RedisStreamError),
}

#[tokio::main]
async fn main() -> Result<(), QueuingError> {
    env_logger::init();

    info!("Starting message queuing");

    if let Ok(redis_domain) = env::var("REDIS_URL") {
        match redis::Client::open(redis_domain.clone()) {
            Ok(redis_client) => {
                let stream_client = RedisStreamClient {
                    client: redis_client.clone(),
                };

                // We don't care for existing group errors
                match stream_client.create_group_and_stream() {
                    _ => (),
                }

                loop {
                    match redis_client.get_connection() {
                        Ok(mut con) => {
                            // Make the current connection connect to the messages database
                            let _: Result<(), redis::RedisError> =
                                redis::cmd("SELECT").arg(MESSAGE_DATABASE).query(&mut con);

                            let mut channel: Channel = get_rss_data().await?;
                            channel.items.reverse();

                            for item in channel.items {
                                if let Some(link) = item.link {
                                    let id = link.clone();
                                    let res: i64 = redis::cmd("EXISTS").arg(&id).query(&mut con)?;

                                    if res == 0 {
                                        let category = match item.categories.first() {
                                            Some(c) => c.name.clone(),
                                            _ => "".to_string(),
                                        };

                                        let title = match item.title {
                                            Some(t) => t,
                                            _ => "".to_string(),
                                        };

                                        let stream_entry = StreamEntry {
                                            message_id: link.clone(),
                                            link,
                                            category: category.to_lowercase(),
                                            title,
                                        };

                                        match stream_client.add(stream_entry, &mut con) {
                                            Ok(id) => info!("added id: {}", id),
                                            Err(e) => warn!("xadd failed: {}", e),
                                        }
                                    }
                                }
                            }

                            sleep(Duration::from_millis(5000));
                        }
                        Err(_) => error!("Redis connection failed"),
                    };
                }
            }
            _ => error!("Couldn't connect to redis"),
        };
    }

    Ok(())
}
