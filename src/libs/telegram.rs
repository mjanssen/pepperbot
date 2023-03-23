use redis::Client;
use teloxide::{prelude::*, utils::command::BotCommands, RequestError};
use thiserror::Error;

use super::redis::SUBSCRIBER_DATABASE;

#[derive(Error, Debug)]
pub enum BotError {
    #[error("No subscribers found")]
    NoSubscribers,

    #[error(transparent)]
    SendMessageError(#[from] RequestError),
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

pub struct BotCommandService {
    pub bot: Bot,
    pub redis_client: Client,
}

impl BotCommandService {
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Started bot command service");

        let bot = self.bot.clone();
        let redis_client = self.redis_client.clone();

        Command::repl(bot, move |bot, msg, cmd| {
            // redis service clone is required, otherwise we lose the reference
            BotCommandService::answer(bot, msg, cmd, redis_client.clone())
        })
        .await;

        Ok(())
    }

    async fn answer(
        bot: Bot,
        msg: Message,
        cmd: Command,
        redis_client: Client,
    ) -> ResponseResult<()> {
        match cmd {
            Command::Help => {
                bot.send_message(msg.chat.id, Command::descriptions().to_string())
                    .await?
            }
            Command::Start => {
                if let Ok(mut con) = redis_client.get_connection() {
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
                println!("bb");
                if let Ok(mut con) = redis_client.get_connection() {
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

#[derive(Clone)]
pub struct BotMessageService {
    pub bot: Bot,
}

impl BotMessageService {
    pub async fn send_message(
        &self,
        chat_id: String,
        message: String,
    ) -> Result<teloxide::prelude::Message, BotError> {
        Ok(self.bot.send_message(chat_id, message).await?)
    }

    pub async fn get_subscribers(&self, redis_client: Client) -> Result<Vec<String>, BotError> {
        if let Ok(mut con) = redis_client.get_connection() {
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
