use std::time::Duration;

use rss::Channel;
use teloxide::{prelude::*, utils::command::BotCommands};
use thiserror::Error;
use tokio::time::sleep;

use super::{redis::RedisService, rss::get_rss_data};

const SUBSCRIBER_DATABASE: u8 = 0;
const MESSAGE_DATABASE: u8 = 1;

#[derive(Error, Debug)]
enum BotError {
    #[error("No subscribers found")]
    NoSubscribers,
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "List these options with information.")]
    Help,
    #[command(description = "Signup for Pepperbot to receive messages when a new deal is listed.")]
    Start,
    #[command(description = "Cancel subscription for Pepperbot")]
    Stop,
}

struct BotCommandService {
    bot: Bot,
    redis_service: RedisService,
}

impl BotCommandService {
    async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Starting bot command service");

        let bot = self.bot.clone();
        let redis_service = self.redis_service.clone();

        Command::repl(bot, move |bot, msg, cmd| {
            // redis service clone is required, otherwise we lose the reference
            BotCommandService::answer(bot, msg, cmd, redis_service.clone())
        })
        .await;

        Ok(())
    }

    async fn answer(
        bot: Bot,
        msg: Message,
        cmd: Command,
        redis_service: RedisService,
    ) -> ResponseResult<()> {
        match cmd {
            Command::Help => {
                bot.send_message(msg.chat.id, Command::descriptions().to_string())
                    .await?
            }
            Command::Start => {
                if let Ok(mut con) = redis_service.client.get_connection() {
                    // Set correct database first
                    let _: Result<(), redis::RedisError> = redis::cmd("SELECT")
                        .arg(SUBSCRIBER_DATABASE)
                        .query(&mut con);

                    let _: Result<(), redis::RedisError> = redis::cmd("SET")
                        .arg(msg.chat.id.to_string())
                        .arg(1)
                        .query(&mut con);
                }

                bot.send_message(
                    msg.chat.id,
                    "Signup was successful. You will now receive new updates from Pepper",
                )
                .await?
            }
            Command::Stop => {
                if let Ok(mut con) = redis_service.client.get_connection() {
                    // Set correct database first
                    let _: Result<(), redis::RedisError> = redis::cmd("SELECT")
                        .arg(SUBSCRIBER_DATABASE)
                        .query(&mut con);

                    let _deleted_amount: Result<i32, redis::RedisError> = redis::cmd("DEL")
                        .arg(msg.chat.id.to_string())
                        .query(&mut con);
                }

                bot.send_message(
                msg.chat.id,
                "Subscription was stopped successfully. You will now receive new updates from Pepper",
            )
            .await?
            }
        };

        Ok(())
    }
}

pub async fn init_bot_commands(redis_service: RedisService) -> ResponseResult<()> {
    let bot_service = BotCommandService {
        bot: Bot::from_env(),
        redis_service,
    };

    let _ = bot_service.start().await;

    Ok(())
}

#[derive(Clone)]
struct BotMessageService {
    bot: Bot,
}

impl BotMessageService {
    pub async fn send_message(
        &self,
        chat_id: String,
        message: String,
    ) -> Result<teloxide::prelude::Message, Box<dyn std::error::Error>> {
        Ok(self.bot.send_message(chat_id, message).await?)
    }

    pub async fn get_subscribers(
        &self,
        redis_service: RedisService,
    ) -> Result<Vec<String>, BotError> {
        if let Ok(mut con) = redis_service.client.get_connection() {
            let _: Result<(), redis::RedisError> = redis::cmd("SELECT")
                .arg(SUBSCRIBER_DATABASE)
                .query(&mut con);

            let keys: Result<Vec<String>, redis::RedisError> =
                redis::cmd("KEYS").arg("*").query(&mut con);

            if let Ok(subscribers) = keys {
                return Ok(subscribers);
            }
        }

        Err(BotError::NoSubscribers)
    }
}

pub async fn init_bot_item_updates(
    redis_service: RedisService,
) -> Result<(), Box<dyn std::error::Error>> {
    let bot_service = BotMessageService {
        bot: Bot::from_env(),
    };

    // Create endless loop that checks items every x-seconds
    loop {
        get_items_and_notify(redis_service.clone(), bot_service.clone()).await?;
        sleep(Duration::from_millis(5000)).await;
    }
}

async fn get_items_and_notify(
    redis_service: RedisService,
    bot_service: BotMessageService,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(mut con) = redis_service.client.get_connection() {
        // Make the current connection connect to the messages database
        let _: Result<(), redis::RedisError> =
            redis::cmd("SELECT").arg(MESSAGE_DATABASE).query(&mut con);

        let mut channel: Channel = get_rss_data().await?;
        channel.items.reverse();

        for item in channel.items {
            let id = item.link.clone();
            let res: i64 = redis::cmd("EXISTS").arg(&id).query(&mut con)?;

            // If item has not been send yet, send it to all users
            if res == 0 {
                if let Some(link) = item.link {
                    let title: String = match item.title {
                        Some(t) => t,
                        _ => "".to_string(),
                    };

                    let subscribers = bot_service.get_subscribers(redis_service.clone()).await;

                    if let Ok(chat_ids) = subscribers {
                        for chat_id in chat_ids {
                            let _ = bot_service
                                .send_message(chat_id, format!("{}\n{}", title, link))
                                .await;
                        }
                    }
                }

                let _: Result<(), redis::RedisError> =
                    redis::cmd("SET").arg(&id).arg(1).query(&mut con);
            }
        }
    }

    Ok(())
}
