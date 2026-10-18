pub mod rss;
use crate::{
    error::Result,
    models::{MediaItem, Subscription},
};
use async_trait::async_trait;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectorInfo {
    pub platform: &'static str,
    pub implemented: bool,
    pub manual_source: bool,
    pub subscription_import: bool,
    pub authenticated_access: bool,
}

pub fn catalog() -> Vec<ConnectorInfo> {
    ["rss", "youtube", "bluesky", "x", "rednote"]
        .into_iter()
        .map(|platform| ConnectorInfo {
            platform,
            implemented: platform == "rss",
            manual_source: platform == "rss",
            subscription_import: false,
            authenticated_access: false,
        })
        .collect()
}

pub struct FetchResult {
    pub title: String,
    pub items: Vec<MediaItem>,
}

#[async_trait]
pub trait PlatformConnector: Send + Sync {
    fn platform(&self) -> &'static str;
    async fn fetch(&self, source: &Subscription) -> Result<FetchResult>;
}
