use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphqlResponse {
    pub data: Data,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    pub suggestions: Suggestions,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Suggestions {
    pub deal_count: i64,
    pub deals: Vec<Deal>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Deal {
    pub thread_id: String,
    pub thread_type_id: i64,
    pub title_slug: String,
    pub price: f64,
    pub display_price: String,
    pub discount_type: Value,
    pub next_best_price: Value,
    pub percentage: f64,
    pub temperature: f64,
}
