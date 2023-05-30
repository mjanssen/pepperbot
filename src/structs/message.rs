use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum MessageError {
    #[error(transparent)]
    RedisError(#[from] redis::RedisError),

    #[error("No subscribers found")]
    ParseError,
}

#[derive(Serialize)]
pub struct Message {
    pub id: String,
    pub channel: String,
    pub payload: Deal,
}

impl Message {
    pub fn new(payload: Deal) -> Message {
        Message {
            id: Uuid::new_v4().to_string(),
            channel: String::from("deals"),
            payload,
        }
    }
}

#[derive(Serialize)]
pub struct Deal {
    pub id: String,
    pub link: String,
    pub category: String,
    pub title: String,
}

impl Deal {
    pub fn new(link: String, category: String, title: String) -> Deal {
        Deal {
            id: link.clone(),
            link,
            category,
            title,
        }
    }
}
