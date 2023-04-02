use std::collections::HashMap;

use log::info;
use redis::{Client, Commands};
use teloxide::{prelude::*, utils::command::BotCommands, RequestError};
use thiserror::Error;

use crate::libs::category::match_category;

use super::{category::CATEGORIES, redis::SUBSCRIBER_DATABASE};

#[derive(Error, Debug)]
pub enum BotError {
    #[error("No subscribers found")]
    NoSubscribers,

    #[error(transparent)]
    SendMessageError(#[from] RequestError),
}

#[derive(BotCommands, Clone, Debug)]
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
    #[command(
        description = "Signup for one of the Pepper categories. Use /availablecategories to see which categories are available."
    )]
    Categories,
    #[command(description = "List available Pepper categories")]
    AvailableCategories,
}

pub struct BotCommandService {
    pub bot: Bot,
    pub redis_client: Client,
}

impl BotCommandService {
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Started bot command service");

        let bot = self.bot.clone();
        let redis_client = self.redis_client.clone();

        Command::repl(bot, move |bot, msg, cmd| {
            info!("Received command: Command::{:?}", cmd);
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
            Command::Categories => {
                if let Ok(mut con) = redis_client.get_connection() {
                    if let Some(text) = msg.text() {
                        // Set correct database first
                        let _: Result<(), redis::RedisError> = redis::cmd("SELECT")
                            .arg(SUBSCRIBER_DATABASE)
                            .query(&mut con);

                        let message = text.replace("/categories", "");

                        let mut passed_categories: Vec<String> = vec![];
                        for possible_cat in message.split(",") {
                            let trimmed_category = possible_cat.trim();

                            if trimmed_category.len() > 0 {
                                if let Some(pepper_category) = match_category(trimmed_category) {
                                    info!("Matched {} -> {:?}", trimmed_category, pepper_category);
                                    passed_categories.push(pepper_category);
                                }
                            }
                        }

                        // If there are no categories found or set, reset filters
                        if passed_categories.len() == 0 {
                            let _: Result<(), redis::RedisError> = redis::cmd("SET")
                                .arg(msg.chat.id.to_string())
                                .arg(1)
                                .query(&mut con);

                            bot.send_message(
                                msg.chat.id,
                                "No categories found, disabled category filters",
                            )
                            .await?
                        // If there are filters found, set the filters for this user
                        } else {
                            let _: Result<(), redis::RedisError> = redis::cmd("SET")
                                .arg(msg.chat.id.to_string())
                                .arg(passed_categories.join(","))
                                .query(&mut con);

                            bot.send_message(
                                msg.chat.id,
                                format!("Signed up for {}", passed_categories.join(", ")),
                            )
                            .await?
                        }
                    // This user did some magic
                    } else {
                        bot.send_message(
                            msg.chat.id,
                            "Something went wrong with reading your message, please try again.",
                        )
                        .await?
                    }
                } else {
                    bot.send_message(
                        msg.chat.id,
                        "Our service is currently down, please try again later.",
                    )
                    .await?
                }
            }
            Command::AvailableCategories => {
                bot.send_message(
                    msg.chat.id,
                    format!(
                        "The following categories are available for signups: \n\n{}",
                        CATEGORIES.join("\n")
                    ),
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
        Ok(self
            .bot
            .send_message(chat_id, message)
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .await?)
    }

    pub async fn get_subscribers(
        &self,
        redis_client: Client,
    ) -> Result<HashMap<String, Option<Vec<String>>>, BotError> {
        if let Ok(mut con) = redis_client.get_connection() {
            let _: Result<(), redis::RedisError> = redis::cmd("SELECT")
                .arg(SUBSCRIBER_DATABASE)
                .query(&mut con);

            let keys: Result<Vec<String>, redis::RedisError> =
                redis::cmd("KEYS").arg("*").query(&mut con);

            if let Ok(chat_ids) = keys {
                let mut subscribers: HashMap<String, Option<Vec<String>>> = HashMap::new();
                for chat_id in &chat_ids {
                    let categories_string: String = con.get(chat_id).unwrap_or("".to_string());
                    let categories: Vec<String> = categories_string
                        .split(",")
                        .map(|s| s.to_string())
                        .collect();

                    // Fix for initial sign-ups
                    if categories.contains(&"1".to_string()) {
                        subscribers.insert(chat_id.clone(), None);
                    } else {
                        subscribers.insert(chat_id.clone(), Some(categories));
                    }
                }

                return Ok(subscribers);
            }
        }

        Err(BotError::NoSubscribers)
    }
}
