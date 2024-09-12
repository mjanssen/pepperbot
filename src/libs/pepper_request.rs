use axum::http::HeaderMap;
use log::{info, warn};
use reqwest::{
    header::{ACCEPT, CONTENT_TYPE, COOKIE, HOST, ORIGIN, REFERER, USER_AGENT},
    Client, Method, Response,
};
use serde_json::{json, Value};

use crate::structs::graphql_response::GraphqlResponse;

pub struct PepperRequest<'a> {
    base: &'a str,
    graphql_endpoint: &'a str,
    reqwest_client: Client,
}

impl PepperRequest<'_> {
    pub fn new() -> Self {
        PepperRequest {
            base: "https://nl.pepper.com",
            graphql_endpoint: "/graphql",
            reqwest_client: reqwest::Client::new(),
        }
    }

    pub fn get_endpoint(&self) -> String {
        format!("{}{}", self.base, self.graphql_endpoint)
    }

    pub async fn request(
        &self,
        path: &str,
        method: Method,
        body: Option<Value>,
        cookie_headers: Option<Vec<String>>,
    ) -> Option<Response> {
        let url = format!("{}{}", self.base, path);
        let mut headers = HeaderMap::new();

        headers.insert(
            USER_AGENT,
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:129.0) Gecko/20100101 Firefox/129.0"
                .parse()
                .unwrap(),
        );
        headers.insert("X-Requested-With", "XMLHttpRequest".parse().unwrap());
        headers.insert(ORIGIN, self.base.parse().unwrap());
        headers.insert(REFERER, url.parse().unwrap());
        headers.insert(HOST, "nl.pepper.com".parse().unwrap());

        if let Some(cookie_headers) = cookie_headers {
            headers.insert(COOKIE, cookie_headers.join(" ").parse().unwrap());
        }

        match method {
            Method::GET => match self.reqwest_client.get(&url).headers(headers).send().await {
                Ok(res) => {
                    info!("Requested: {}", url);
                    return Some(res);
                }
                Err(e) => {
                    warn!("{}", e);
                    return None;
                }
            },
            Method::POST => match body {
                Some(body) => {
                    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
                    headers.insert(ACCEPT, "application/json".parse().unwrap());

                    match self
                        .reqwest_client
                        .post(&url)
                        .json(&body)
                        .headers(headers)
                        .send()
                        .await
                    {
                        Ok(res) => {
                            info!("Requested: {}", url);
                            return Some(res);
                        }
                        Err(e) => {
                            warn!("{}", e);
                            return None;
                        }
                    }
                }
                None => {
                    warn!("Body is required when doing a post");
                    return None;
                }
            },
            _ => None,
        }
    }

    pub async fn get_cookie_headers(&self) -> Option<Vec<String>> {
        if let Some(response) = self.request("/", Method::GET, None, None).await {
            let cookie_headers = response
                .headers()
                .get_all(reqwest::header::SET_COOKIE)
                .iter()
                .map(|v| {
                    v.to_str()
                        .unwrap_or("")
                        .split(';')
                        .next()
                        .map(|s| s.trim().to_string() + ";")
                        .unwrap_or_default()
                })
                .collect::<Vec<String>>();

            return Some(cookie_headers);
        }

        None
    }

    pub async fn graphql(&self, search: &str) -> Option<GraphqlResponse> {
        if let Some(cookie_headers) = self.get_cookie_headers().await {
            let body = json!({
              "query": "query searchSuggestions(
                  $query: String
                  $dealsLimit: Int
                ) {
                  suggestions: searchSuggestions(
                    query: $query
                    dealsLimit: $dealsLimit
                  ) {
                    dealCount
                    deals {
                      ...threadFragment
                    }
                  }
                }

                fragment threadFragment on Thread {
                  threadId
                  threadTypeId
                  titleSlug
                  price
                  displayPrice
                  discountType
                  nextBestPrice
                  percentage
                  temperature
                }",
              "variables": {
                "query": search,
                "dealsLimit": 10
              }
            });

            if let Some(response) = self
                .request(
                    self.graphql_endpoint,
                    Method::POST,
                    Some(body),
                    Some(cookie_headers),
                )
                .await
            {
                match response.json::<GraphqlResponse>().await {
                    Ok(json) => return Some(json),
                    Err(e) => {
                        warn!("{:?}", e);
                        return None;
                    }
                }
            }
        }

        None
    }
}
