use rss::Channel;
use thiserror::Error;

static RSS_URL: &str = "https://nl.pepper.com/rss/nieuw";

#[derive(Error, Debug)]
pub enum RSSError {
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),

    #[error(transparent)]
    RSSError(#[from] rss::Error),
}

pub async fn get_rss_data() -> Result<Channel, RSSError> {
    let feed = reqwest::get(RSS_URL).await?.bytes().await?;

    let channel = Channel::read_from(&feed[..])?;

    Ok(channel)
}
