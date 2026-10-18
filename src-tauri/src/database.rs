use crate::{
    connectors::FetchResult,
    error::{Error, Result},
    models::{FeedFilter, MediaItem, Subscription},
};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};
use std::path::Path;

pub async fn open(path: &Path) -> Result<SqlitePool> {
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);
    let pool = SqlitePoolOptions::new()
        .max_connections(4)
        .connect_with(options)
        .await?;
    migrate(&pool).await?;
    Ok(pool)
}
pub async fn migrate(pool: &SqlitePool) -> Result<()> {
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}
pub async fn subscriptions(pool: &SqlitePool) -> Result<Vec<Subscription>> {
    Ok(
        sqlx::query_as("SELECT * FROM subscriptions ORDER BY title, id")
            .fetch_all(pool)
            .await?,
    )
}
pub async fn add(pool: &SqlitePool, url: &str) -> Result<Subscription> {
    let url = crate::connectors::rss::validate_url(url)?;
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO subscriptions(id, platform, url, title) VALUES (?, 'rss', ?, ?) ON CONFLICT(url) DO NOTHING")
        .bind(id).bind(&url).bind(&url).execute(pool).await?;
    Ok(sqlx::query_as("SELECT * FROM subscriptions WHERE url = ?")
        .bind(url)
        .fetch_one(pool)
        .await?)
}
pub async fn remove(pool: &SqlitePool, id: &str) -> Result<()> {
    sqlx::query("DELETE FROM subscriptions WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
pub async fn feed(pool: &SqlitePool, filter: FeedFilter) -> Result<Vec<MediaItem>> {
    Ok(sqlx::query_as("SELECT i.*, COALESCE(s.is_read, 0) AS is_read FROM items i JOIN subscriptions sub ON sub.id = i.source_id LEFT JOIN item_states s ON s.item_id = i.id WHERE (? IS NULL OR i.source_id = ?) AND (? = 0 OR COALESCE(s.is_read, 0) = 0) ORDER BY COALESCE(i.published_at, i.fetched_at) DESC, i.id DESC LIMIT ? OFFSET ?")
        .bind(&filter.source_id).bind(&filter.source_id).bind(filter.unread_only)
        .bind(filter.limit.unwrap_or(50).clamp(1, 200)).bind(filter.offset.unwrap_or(0))
        .fetch_all(pool).await?)
}
pub async fn set_read(pool: &SqlitePool, id: &str, is_read: bool) -> Result<()> {
    if !sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM items WHERE id = ?)")
        .bind(id)
        .fetch_one(pool)
        .await?
    {
        return Err(Error::Invalid("Item no longer exists".into()));
    }
    sqlx::query("INSERT INTO item_states(item_id, is_read) VALUES (?, ?) ON CONFLICT(item_id) DO UPDATE SET is_read = excluded.is_read")
        .bind(id).bind(is_read).execute(pool).await?;
    Ok(())
}
/// Content and success marker commit together; read state lives separately.
pub async fn ingest(pool: &SqlitePool, source: &Subscription, result: FetchResult) -> Result<()> {
    let mut tx = pool.begin().await?;
    for item in result.items {
        if item.source_id != source.id {
            return Err(Error::Invalid("Connector source mismatch".into()));
        }
        sqlx::query("INSERT INTO items(id, source_id, external_id, title, url, published_at, fetched_at) VALUES (?, ?, ?, ?, ?, ?, ?) ON CONFLICT(source_id, external_id) DO UPDATE SET title=excluded.title, url=excluded.url, published_at=excluded.published_at")
            .bind(item.id).bind(item.source_id).bind(item.external_id).bind(item.title).bind(item.url)
            .bind(item.published_at).bind(item.fetched_at).execute(&mut *tx).await?;
    }
    sqlx::query("UPDATE subscriptions SET title=?, last_synced_at=?, last_error=NULL WHERE id=?")
        .bind(result.title)
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(&source.id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    async fn memory() -> SqlitePool {
        let opts = SqliteConnectOptions::from_str("sqlite::memory:")
            .unwrap()
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(opts)
            .await
            .unwrap();
        migrate(&pool).await.unwrap();
        pool
    }
    #[tokio::test]
    async fn dedup_sort_filter_read_and_unsubscribe() {
        let pool = memory().await;
        migrate(&pool).await.unwrap();
        let source = add(&pool, "https://example.com/feed").await.unwrap();
        assert_eq!(
            add(&pool, "https://example.com/feed").await.unwrap().id,
            source.id
        );
        for _ in 0..2 {
            let result = crate::connectors::rss::parse(
                include_bytes!("../../tests/fixtures/rss.xml"),
                &source,
            )
            .unwrap();
            ingest(&pool, &source, result).await.unwrap();
        }
        let items = feed(&pool, FeedFilter::default()).await.unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].external_id, "one");
        set_read(&pool, &items[0].id, true).await.unwrap();
        ingest(
            &pool,
            &source,
            crate::connectors::rss::parse(include_bytes!("../../tests/fixtures/rss.xml"), &source)
                .unwrap(),
        )
        .await
        .unwrap();
        let unread = feed(
            &pool,
            FeedFilter {
                unread_only: true,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert_eq!(unread.len(), 1);
        assert_eq!(unread[0].external_id, "two");
        assert!(feed(
            &pool,
            FeedFilter {
                source_id: Some("absent".into()),
                ..Default::default()
            }
        )
        .await
        .unwrap()
        .is_empty());
        set_read(&pool, &items[0].id, false).await.unwrap();
        assert_eq!(feed(&pool, FeedFilter::default()).await.unwrap().len(), 2);
        remove(&pool, &source.id).await.unwrap();
        assert!(feed(&pool, FeedFilter::default()).await.unwrap().is_empty());
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM item_states")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 0);
    }
    #[tokio::test]
    async fn persists_on_reopen_and_rolls_back_bad_batch() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.db");
        let pool = open(&path).await.unwrap();
        let source = add(&pool, "https://example.com/feed").await.unwrap();
        let mut result =
            crate::connectors::rss::parse(include_bytes!("../../tests/fixtures/rss.xml"), &source)
                .unwrap();
        result.items[1].source_id = "wrong".into();
        assert!(ingest(&pool, &source, result).await.is_err());
        assert!(feed(&pool, FeedFilter::default()).await.unwrap().is_empty());
        ingest(
            &pool,
            &source,
            crate::connectors::rss::parse(include_bytes!("../../tests/fixtures/rss.xml"), &source)
                .unwrap(),
        )
        .await
        .unwrap();
        let item = feed(&pool, FeedFilter::default()).await.unwrap().remove(0);
        set_read(&pool, &item.id, true).await.unwrap();
        pool.close().await;
        let reopened = open(&path).await.unwrap();
        assert!(feed(&reopened, FeedFilter::default()).await.unwrap()[0].is_read);
    }
}
