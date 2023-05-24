use reqwest::header::{ACCEPT, ACCEPT_LANGUAGE, USER_AGENT};
use rss::Channel;
use thiserror::Error;

static RSS_URL: &str = "https://nl.pepper.com/rss/nieuw";

#[derive(Error, Debug)]
pub enum RSSError {
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),

    #[error(transparent)]
    RSSError(#[from] rss::Error),

    #[error(transparent)]
    RedisError(#[from] redis::RedisError),
}

pub async fn get_rss_data() -> Result<Channel, RSSError> {
    let client = reqwest::Client::new();
    let feed = client
        .get(RSS_URL)
        .header(USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.36")
        .header(ACCEPT, "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9")
        .header(ACCEPT_LANGUAGE, "en-GB,en;q=0.6")
        .header("sec-ch-ua", "\"Brave\";v=\"113\", \"Chromium\";v=\"113\", \"Not-A.Brand\";v=\"24\"")
        .header("sec-ch-ua-mobile", "?0")
        .header("sec-ch-ua-platform", "\"macOS\"")
        .header("sec-fetch-site", "none")
        .header("sec-fetch-mod", "")
        .header("sec-fetch-user", "?1")
        .header("sec-fetch-mode", "navigate")
        .send()
        .await?
        .bytes()
        .await?;

    let channel = Channel::read_from(&feed[..])?;

    Ok(channel)
}
