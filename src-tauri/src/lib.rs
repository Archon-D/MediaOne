pub mod auth;
pub mod connectors;
pub mod database;
pub mod error;
pub mod ingestion;
pub mod models;

use tauri::{Manager, State};
use tauri_plugin_opener::OpenerExt;
use models::{FeedFilter, MediaItem, Subscription, SyncReport};
struct AppState { pool: sqlx::SqlitePool, ingestion: ingestion::Ingestion }

#[tauri::command]
async fn get_feed(state: State<'_, AppState>, filter: FeedFilter) -> Result<Vec<MediaItem>, String> {
    database::feed(&state.pool, filter).await.map_err(|e| e.to_string())
}
#[tauri::command]
async fn list_subscriptions(state: State<'_, AppState>) -> Result<Vec<Subscription>, String> {
    database::subscriptions(&state.pool).await.map_err(|e| e.to_string())
}
#[tauri::command]
async fn add_subscription(state: State<'_, AppState>, url: String) -> Result<Subscription, String> {
    database::add(&state.pool, &url).await.map_err(|e| e.to_string())
}
#[tauri::command]
async fn remove_subscription(state: State<'_, AppState>, id: String) -> Result<(), String> {
    // Serialize removal with refresh so a removed source cannot be re-ingested.
    let _guard = state.ingestion.lock.lock().await;
    database::remove(&state.pool, &id).await.map_err(|e| e.to_string())
}
#[tauri::command]
async fn set_read_state(state: State<'_, AppState>, id: String, is_read: bool) -> Result<(), String> {
    database::set_read(&state.pool, &id, is_read).await.map_err(|e| e.to_string())
}
#[tauri::command]
async fn refresh_feeds(state: State<'_, AppState>) -> Result<Vec<SyncReport>, String> {
    state.ingestion.refresh(&state.pool).await.map_err(|e| e.to_string())
}
#[tauri::command]
fn list_connectors() -> Vec<connectors::ConnectorInfo> { connectors::catalog() }
#[tauri::command]
async fn list_accounts(state: State<'_, AppState>) -> Result<Vec<auth::Account>, String> {
    sqlx::query_as("SELECT id, platform, display_name, status FROM connected_accounts").fetch_all(&state.pool).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn open_original(app: tauri::AppHandle, state: State<'_, AppState>, id: String) -> Result<(), String> {
    let url: Option<String> = sqlx::query_scalar("SELECT url FROM items WHERE id = ?")
        .bind(id).fetch_optional(&state.pool).await.map_err(|e| e.to_string())?.flatten();
    let url = url.ok_or("Article has no original URL")?;
    let validated = connectors::rss::validate_url(&url).map_err(|e| e.to_string())?;
    app.opener().open_url(validated, None::<&str>).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let pool = tauri::async_runtime::block_on(database::open(&dir.join("mediaone.db")))?;
            app.manage(AppState { pool, ingestion: ingestion::Ingestion { lock: tokio::sync::Mutex::new(()), rss: connectors::rss::RssConnector::new()? } });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_feed, list_subscriptions, add_subscription, remove_subscription, set_read_state, refresh_feeds, list_connectors, list_accounts, open_original])
        .run(tauri::generate_context!())
        .expect("Unable to start MediaOne");
}
