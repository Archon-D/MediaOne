//! Session extension points only: no login or secret persistence is implemented.
use async_trait::async_trait;
use serde::Serialize;

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: String,
    pub platform: String,
    pub display_name: String,
    pub status: String,
}

/// Implement with platform Keychain/Keystore before enabling authenticated connectors.
/// Secret bytes must stay in Rust and must never enter SQLite, logs, or IPC responses.
#[async_trait]
pub trait CredentialVault: Send + Sync {
    async fn load(&self, reference: &str) -> Result<Vec<u8>, String>;
    async fn store(&self, reference: &str, secret: &[u8]) -> Result<(), String>;
    async fn delete(&self, reference: &str) -> Result<(), String>;
}

pub struct SessionManager<V: CredentialVault> { pub vault: V }
impl<V: CredentialVault> SessionManager<V> {
    pub async fn disconnect(&self, credential_ref: &str) -> Result<(), String> {
        self.vault.delete(credential_ref).await
    }
}
