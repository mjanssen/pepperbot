pub mod libs;

use std::env;

use libs::telegram::BotCommandService;
use teloxide::Bot;

#[tokio::main]
async fn main() -> () {
    if let Ok(redis_domain) = env::var("REDIS_URL") {
        match redis::Client::open(redis_domain.clone()) {
            Ok(redis_client) => {
                let bot_service = BotCommandService {
                    bot: Bot::from_env(),
                    redis_client,
                };

                let _ = bot_service.start().await;
            }
            Err(_) => panic!("Could not connect to redis"),
        }
    }

    ()
}
