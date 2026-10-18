use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Subscription {
    pub id: String,
    pub platform: String,
    pub url: String,
    pub title: String,
    pub last_synced_at: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct MediaItem {
    pub id: String,
    pub source_id: String,
    pub external_id: String,
    pub title: String,
    pub url: Option<String>,
    pub published_at: Option<String>,
    pub fetched_at: String,
    pub is_read: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedFilter {
    pub source_id: Option<String>,
    #[serde(default)]
    pub unread_only: bool,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncReport {
    pub source_id: String,
    pub item_count: usize,
    pub error: Option<String>,
}
