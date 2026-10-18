use sqlx::SqlitePool;
use tokio::sync::Mutex;
use crate::{connectors::{rss::RssConnector, PlatformConnector}, database, error::{Error, Result}, models::SyncReport};

pub struct Ingestion {
    pub lock: Mutex<()>,
    pub rss: RssConnector,
}
impl Ingestion {
    /// One foreground batch per device; sources fail independently. No DB lock spans HTTP.
    pub async fn refresh(&self, pool: &SqlitePool) -> Result<Vec<SyncReport>> {
        let _guard = self.lock.try_lock().map_err(|_| Error::Invalid("Refresh already in progress".into()))?;
        let mut reports = Vec::new();
        for source in database::subscriptions(pool).await? {
            let result = if source.platform == self.rss.platform() {
                match self.rss.fetch(&source).await {
                    Ok(batch) => { let n = batch.items.len(); database::ingest(pool, &source, batch).await.map(|_| n) }
                    Err(e) => Err(e),
                }
            } else { Err(Error::Invalid("Connector is not implemented".into())) };
            let (item_count, error) = match result {
                Ok(n) => (n, None),
                Err(e) => {
                    // Never save network error strings containing potentially sensitive feed URLs.
                    let message = match e { Error::Network(_) => "Feed request failed (network or HTTP status)".to_string(), other => other.to_string() };
                    sqlx::query("UPDATE subscriptions SET last_error=? WHERE id=?").bind(&message).bind(&source.id).execute(pool).await?;
                    (0, Some(message))
                }
            };
            reports.push(SyncReport { source_id: source.id, item_count, error });
        }
        Ok(reports)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};

    #[tokio::test]
    async fn fetches_http_and_isolates_source_failures() {
        let server = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = server.local_addr().unwrap();
        let handle = std::thread::spawn(move || {
            for _ in 0..2 {
                let (mut socket, _) = server.accept().unwrap();
                socket.set_read_timeout(Some(std::time::Duration::from_secs(5))).unwrap();
                let mut request = [0u8; 2048];
                let size = socket.read(&mut request).unwrap();
                let request = String::from_utf8_lossy(&request[..size]);
                let (status, body) = if request.starts_with("GET /good ") {
                    ("200 OK", include_str!("../../tests/fixtures/rss.xml"))
                } else { ("503 Service Unavailable", "unavailable") };
                write!(socket, "HTTP/1.1 {status}\r\nContent-Type: application/rss+xml\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
            }
        });
        let dir = tempfile::tempdir().unwrap();
        let pool = database::open(&dir.path().join("test.db")).await.unwrap();
        let good = database::add(&pool, &format!("http://{address}/good")).await.unwrap();
        let bad = database::add(&pool, &format!("http://{address}/bad")).await.unwrap();
        let ingestion = Ingestion { lock: Mutex::new(()), rss: RssConnector::new().unwrap() };
        let reports = ingestion.refresh(&pool).await.unwrap();
        assert_eq!(reports.len(), 2);
        assert!(reports.iter().find(|r| r.source_id == bad.id).unwrap().error.is_some());
        assert_eq!(reports.iter().find(|r| r.source_id == good.id).unwrap().item_count, 2);
        assert_eq!(database::feed(&pool, Default::default()).await.unwrap().len(), 2);
        let sources = database::subscriptions(&pool).await.unwrap();
        assert!(sources.iter().find(|s| s.id == bad.id).unwrap().last_synced_at.is_none());
        assert!(sources.iter().find(|s| s.id == good.id).unwrap().last_synced_at.is_some());
        handle.join().unwrap();
    }
}
