use serde::{Deserialize, Serialize};
use thiserror::Error;

pub static LIST_NAME: &str = "deals";

#[derive(Debug, Error)]
pub enum MessageError {
    #[error(transparent)]
    RedisError(#[from] redis::RedisError),

    #[error("No subscribers found")]
    ParseError,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub id: String,
    pub list: String,
    pub payload: Deal,
}

impl Message {
    pub fn new(payload: Deal) -> Message {
        Message {
            id: payload.link.clone(),
            list: String::from(LIST_NAME),
            payload,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Deal {
    pub link: String,
    pub category: String,
    pub title: String,
}

impl Deal {
    pub fn new(link: String, category: String, title: String) -> Deal {
        Deal {
            link,
            category,
            title,
        }
    }
}
