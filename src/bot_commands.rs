pub mod libs;
pub mod structs;

use log::{info, warn};

use libs::variable::get_environment_variable;
use libs::version::print_version;
use libs::telegram::BotCommandService;
use teloxide::Bot;

#[tokio::main]
async fn main() {
    env_logger::init();
    print_version();

    info!("Starting bot commands service");

    let redis_url = get_environment_variable("REDIS_URL");

    match redis::Client::open(redis_url.clone()) {
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
