mod libs;

use libs::{
    redis::{get_redis_service, RedisError},
    rss::RSSError,
    telegram::{init_bot_commands, init_bot_item_updates},
};

use thiserror::Error;

#[derive(Error, Debug)]
enum ApplicationError {
    #[error(transparent)]
    ReqwestError(#[from] RSSError),

    #[error(transparent)]
    RedisServiceError(#[from] RedisError),
}

#[tokio::main]
async fn main() {
    match get_redis_service() {
        Ok(redis_service) => {
            let commands_redis = redis_service.clone();
            tokio::spawn(async {
                let _ = init_bot_commands(commands_redis).await;
            });

            tokio::spawn(async {
                let _ = init_bot_item_updates(redis_service).await;
            });

            loop {}
        }
        Err(e) => println!("{:?}", e),
    }
}
