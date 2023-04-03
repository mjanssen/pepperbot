use log::info;
use redis::Client;
use std::env;
use teloxide::{prelude::*, utils::command::BotCommands, RequestError};
use thiserror::Error;

use crate::libs::category::match_category;

use super::{
    category::CATEGORIES,
    redis::{get_subscribers, set_config, Config, Database},
};

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
    #[command(description = "List these options with information")]
    Help,
    #[command(description = "Signup for Pepperbot to receive messages when a new deal is listed")]
    Start,
    #[command(description = "Cancel subscription for Pepperbot")]
    Stop,
    #[command(
        description = "Signup for one of the Pepper categories. Only accepts comma-separated categories"
    )]
    Categories,
    #[command(
        rename = "available_categories",
        description = "List available Pepper categories"
    )]
    AvailableCategories,
    #[command(
        rename = "stop_bot",
        description = "Admin - Stop bot from sending messages"
    )]
    AdminStopBot,
    #[command(
        rename = "start_bot",
        description = "Admin - Allow bot to send messages again"
    )]
    AdminStartBot,
    #[command(
        rename = "broadcast",
        description = "Admin - Broadcast to all subscribed users"
    )]
    AdminBroadcast,
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
            Self::answer(bot, msg, cmd, redis_client.clone())
        })
        .await;

        Ok(())
    }

    fn is_admin(chat_id: &String) -> bool {
        if let Ok(admin_chat_id) = env::var("ADMIN_CHAT_ID") {
            if admin_chat_id.eq(chat_id) {
                // Admin command by admin
                return true;
            }
        }

        info!("Admin command executed by non-admin {}", chat_id);

        return false;
    }

    async fn send_message(bot: &Bot, chat_id: String, message: &str) {
        match bot
            .send_message(chat_id, message.replace("/admin_broadcast", "").trim())
            .await
        {
            Ok(_) => (),
            Err(e) => info!("Message failed sending {}", e),
        };
    }

    async fn answer(
        bot: Bot,
        msg: Message,
        cmd: Command,
        redis_client: Client,
    ) -> Result<(), RequestError> {
        match cmd {
            Command::AdminStopBot => {
                if Self::is_admin(&msg.chat.id.to_string()) {
                    if let Ok(mut con) = redis_client.get_connection() {
                        let _ = set_config(&mut con, Config::OperationalKey, 0);

                        Self::send_message(&bot, msg.chat.id.to_string(), "Stopped bot").await;

                        return Ok(());
                    }
                }

                Ok::<(), RequestError>(())
            }
            Command::AdminStartBot => {
                if Self::is_admin(&msg.chat.id.to_string()) {
                    if let Ok(mut con) = redis_client.get_connection() {
                        let _ = set_config(&mut con, Config::OperationalKey, 1);

                        Self::send_message(&bot, msg.chat.id.to_string(), "Started bot").await;

                        return Ok(());
                    }
                }

                Ok(())
            }
            Command::AdminBroadcast => {
                if Self::is_admin(&msg.chat.id.to_string()) {
                    let message = msg.text().unwrap_or("");

                    let subscribers = get_subscribers(redis_client).await;

                    if let Ok(subs) = subscribers {
                        for (chat_id, _) in subs {
                            Self::send_message(
                                &bot,
                                chat_id,
                                message.replace("/admin_broadcast", "").trim(),
                            )
                            .await;
                        }
                    }
                }

                Ok(())
            }
            Command::Help => {
                if let Ok(admin_chat) = env::var("ADMIN_CHAT_ID") {
                    let cmds_string = Command::descriptions().to_string();

                    let commands: Vec<&str> = cmds_string
                        .split("\n")
                        .filter(|l| {
                            if admin_chat.eq(&msg.chat.id.to_string()) {
                                return true;
                            };

                            l.to_lowercase().contains("admin") == false
                        })
                        .collect();

                    Self::send_message(&bot, msg.chat.id.to_string(), commands.join("\n").as_str())
                        .await;

                    return Ok(());
                }

                Ok(())
            }
            Command::Start => {
                if let Ok(mut con) = redis_client.get_connection() {
                    // Set correct database first
                    let _: Result<(), redis::RedisError> = redis::cmd("SELECT")
                        .arg(Database::SUBSCRIBER as u8)
                        .query(&mut con);

                    let _: Result<(), redis::RedisError> = redis::cmd("SET")
                        .arg(msg.chat.id.to_string())
                        .arg(1)
                        .query(&mut con);
                }

                Self::send_message(
                    &bot,
                    msg.chat.id.to_string(),
                    "Signup was successful. You will now receive new updates from Pepper",
                )
                .await;

                Ok(())
            }
            Command::Stop => {
                if let Ok(mut con) = redis_client.get_connection() {
                    // Set correct database first
                    let _: Result<(), redis::RedisError> = redis::cmd("SELECT")
                        .arg(Database::SUBSCRIBER as u8)
                        .query(&mut con);

                    let _deleted_amount: Result<i32, redis::RedisError> = redis::cmd("DEL")
                        .arg(msg.chat.id.to_string())
                        .query(&mut con);

                    Self::send_message(
                        &bot,
                        msg.chat.id.to_string(),
                        "Subscription was stopped successfully. You will now receive new updates from Pepper",
                    ).await;
                }

                Ok(())
            }
            Command::Categories => {
                if let Ok(mut con) = redis_client.get_connection() {
                    if let Some(text) = msg.text() {
                        // Set correct database first
                        let _: Result<(), redis::RedisError> = redis::cmd("SELECT")
                            .arg(Database::SUBSCRIBER as u8)
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

                            Self::send_message(
                                &bot,
                                msg.chat.id.to_string(),
                                "No categories passed, disabled your category filters",
                            )
                            .await;

                        // If there are filters found, set the filters for this user
                        } else {
                            let _: Result<(), redis::RedisError> = redis::cmd("SET")
                                .arg(msg.chat.id.to_string())
                                .arg(passed_categories.join(","))
                                .query(&mut con);

                            Self::send_message(
                                &bot,
                                msg.chat.id.to_string(),
                                format!("Signed up for {}", passed_categories.join(", ")).as_str(),
                            )
                            .await;
                        }
                    // This user did some magic
                    } else {
                        Self::send_message(
                            &bot,
                            msg.chat.id.to_string(),
                            "Something went wrong with reading your message, please try again.",
                        )
                        .await;
                    }
                } else {
                    Self::send_message(
                        &bot,
                        msg.chat.id.to_string(),
                        "Our service is currently down, please try again later.",
                    )
                    .await;
                }

                Ok(())
            }
            Command::AvailableCategories => {
                Self::send_message(
                    &bot,
                    msg.chat.id.to_string(),
                    format!(
                        "The following categories are available for signups: \n\n{}",
                        CATEGORIES.join("\n")
                    )
                    .as_str(),
                )
                .await;

                Ok(())
            }
        }?;

        Ok(())
    }
}

#[derive(Clone)]
pub struct BotMessageService {
    pub bot: Bot,
}

impl BotMessageService {
    pub async fn send_message(&self, chat_id: String, message: String) -> Result<(), BotError> {
        match self
            .bot
            .send_message(chat_id, message)
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                info!("Message failed sending {}", e);
                Err(BotError::SendMessageError(e))
            }
        }
    }
}
