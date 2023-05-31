pub mod libs;
pub mod structs;

use libs::redis_stream_client::RedisStreamError;
use libs::rss::RSSError;
use log::{error, info};
use rss::Channel;
use std::env;
use std::thread::sleep;
use std::time::Duration;
use structs::message::Message;

use crate::libs::redis::{publish_message, Database};
use crate::libs::rss::get_rss_data;
use crate::libs::version::print_version;
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
    print_version();

    info!("Starting message queuing service");

    if let Ok(redis_domain) = env::var("REDIS_URL") {
        match redis::Client::open(redis_domain.clone()) {
            Ok(redis_client) => {
                loop {
                    match redis_client.get_connection() {
                        Ok(mut con) => {
                            // Make the current connection connect to the messages database
                            let _: Result<(), redis::RedisError> = redis::cmd("SELECT")
                                .arg(Database::MESSAGE as u8)
                                .query(&mut con);

                            let mut channel: Channel = get_rss_data().await?;
                            channel.items.reverse();

                            for item in channel.items {
                                if let Some(link) = item.link {
                                    let id = link.clone();
                                    let res: i64 = redis::cmd("EXISTS").arg(&id).query(&mut con)?;

                                    if res.eq(&1) {
                                        continue;
                                    }

                                    let category = match item.categories.first() {
                                        Some(c) => c.name.clone(),
                                        _ => "".to_string(),
                                    };

                                    let title = match item.title {
                                        Some(t) => t,
                                        _ => "".to_string(),
                                    };

                                    let message = Message::new(structs::message::Deal::new(
                                        link, category, title,
                                    ));

                                    if let Err(e) = publish_message(redis_domain.clone(), message) {
                                        error!("Adding to redis failed {:?}", e);
                                    };
                                }
                            }

                            // Sleep for a while
                            sleep(Duration::from_secs(300));
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
