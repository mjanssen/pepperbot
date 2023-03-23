mod libs;

use std::{env, thread::sleep, time::Duration};

use libs::{
    redis::{get_redis_service, RedisError},
    rss::{get_rss_data, run_message_queuing, RSSError},
    telegram::{init_bot_commands, init_bot_item_updates, run_message_consumer},
};

use redis::{streams::StreamMaxlen, Commands};
use rss::Channel;
use thiserror::Error;

#[derive(Error, Debug)]
enum ApplicationError {
    #[error(transparent)]
    RSSError(#[from] RSSError),

    #[error(transparent)]
    RedisServiceError(#[from] RedisError),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tokio::spawn(async {
        let _ = run_message_queuing().await;
    });

    tokio::spawn(async {
        let _ = run_message_consumer().await;
    });

    match get_redis_service() {
        Ok(redis_service) => {
            // let commands_redis = redis_service.clone();
            // tokio::spawn(async {
            //     let _ = init_bot_commands(commands_redis).await;
            // });

            // tokio::spawn(async {
            //     let _ = init_bot_item_updates(redis_service).await;
            // });

            //             loop {}
            //
        }
        Err(e) => println!("{:?}", e),
    }

    loop {}
}
