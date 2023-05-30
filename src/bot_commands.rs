pub mod libs;
pub mod structs;

use log::{info, warn};
use std::env;

use libs::telegram::BotCommandService;
use teloxide::Bot;

use crate::libs::version::print_version;

#[tokio::main]
async fn main() {
    env_logger::init();
    print_version();

    info!("Starting bot commands service");

    if let Ok(redis_domain) = env::var("REDIS_URL") {
        match redis::Client::open(redis_domain.clone()) {
            Ok(redis_client) => {
                let bot_service = BotCommandService {
                    bot: Bot::from_env(),
                    redis_client,
                };

                let _ = bot_service.start().await;
            }
            Err(_) => warn!("Could not connect to redis"),
        }
    }
}
