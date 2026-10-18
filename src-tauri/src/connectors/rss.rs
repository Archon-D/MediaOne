use async_trait::async_trait;
use chrono::{SecondsFormat, Utc};
use futures_util::StreamExt;
use std::time::Duration;
use url::Url;
use crate::{error::{Error, Result}, models::{MediaItem, Subscription}};
use super::{FetchResult, PlatformConnector};

const MAX_BYTES: usize = 5 * 1024 * 1024;
pub fn validate_url(value: &str) -> Result<String> {
    let url = Url::parse(value.trim()).map_err(|_| Error::Invalid("Enter a valid feed URL".into()))?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none()
        || !url.username().is_empty() || url.password().is_some() {
        return Err(Error::Invalid("Feed URL must use HTTP(S) without embedded credentials".into()));
    }
    let mut url = url;
    url.set_fragment(None);
    Ok(url.to_string())
}

pub struct RssConnector { client: reqwest::Client }
impl RssConnector {
    pub fn new() -> Result<Self> {
        Ok(Self { client: reqwest::Client::builder()
            .timeout(Duration::from_secs(20))
            .redirect(reqwest::redirect::Policy::limited(5))
            .user_agent("MediaOne/0.1") .build()? })
    }
}

#[async_trait]
impl PlatformConnector for RssConnector {
    fn platform(&self) -> &'static str { "rss" }
    async fn fetch(&self, source: &Subscription) -> Result<FetchResult> {
        let url = validate_url(&source.url)?;
        let response = self.client.get(url).send().await?.error_for_status()?;
        if response.content_length().is_some_and(|n| n > MAX_BYTES as u64) {
            return Err(Error::Invalid("Feed exceeds 5 MiB limit".into()));
        }
        let mut bytes = Vec::new();
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            if bytes.len() + chunk.len() > MAX_BYTES { return Err(Error::Invalid("Feed exceeds 5 MiB limit".into())); }
            bytes.extend_from_slice(&chunk);
        }
        parse(&bytes, source)
    }
}

pub fn parse(bytes: &[u8], source: &Subscription) -> Result<FetchResult> {
    let feed = feed_rs::parser::Builder::new().base_uri(Some(&source.url)).build().parse(bytes)
        .map_err(|_| Error::Invalid("Response is not a valid RSS/Atom feed".into()))?;
    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let items = feed.entries.into_iter().map(|entry| {
        let link = entry.links.iter().find(|l| l.rel.as_deref().is_none_or(|r| r == "alternate"))
            .and_then(|l| Url::parse(&source.url).ok()?.join(&l.href).ok())
            .filter(|u| matches!(u.scheme(), "http" | "https"))
            .map(|u| u.to_string());
        MediaItem {
            id: format!("{}:{}", source.id, entry.id),
            source_id: source.id.clone(), external_id: entry.id,
            title: entry.title.map(|t| t.content).unwrap_or_else(|| "Untitled".into()),
            url: link,
            published_at: entry.published.or(entry.updated).map(|t| t.to_rfc3339_opts(SecondsFormat::Millis, true)),
            fetched_at: now.clone(), is_read: false,
        }
    }).collect();
    Ok(FetchResult { title: feed.title.map(|t| t.content).unwrap_or_else(|| source.title.clone()), items })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn source() -> Subscription { Subscription { id: "s".into(), platform: "rss".into(), url: "https://example.com/feed".into(), title: "Example".into(), last_synced_at: None, last_error: None } }
    #[test]
    fn parses_rss_and_atom() {
        for xml in [include_bytes!("../../../tests/fixtures/rss.xml").as_slice(), include_bytes!("../../../tests/fixtures/atom.xml").as_slice()] {
            let result = parse(xml, &source()).unwrap();
            assert_eq!(result.items.len(), 2);
            assert!(result.items[0].published_at.is_some());
            assert_eq!(result.items[0].id, parse(xml, &source()).unwrap().items[0].id);
        }
    }
    #[test]
    fn rejects_invalid_inputs_and_unsafe_links() {
        assert!(parse(b"garbage", &source()).is_err());
        for url in ["file:///etc/passwd", "javascript:alert(1)", "https://user:pass@example.com"] { assert!(validate_url(url).is_err()); }
        let parsed = parse(br#"<feed xmlns="http://www.w3.org/2005/Atom"><id>f</id><title>f</title><updated>2026-01-01T00:00:00Z</updated><entry><id>e</id><title>x</title><link href="javascript:alert(1)"/></entry></feed>"#, &source()).unwrap();
        assert!(parsed.items[0].url.is_none());
    }
}
