# Local data model

SQLite is opened with foreign keys and WAL enabled. SQLx applies embedded versioned migrations; never edit an already-released migration, add a new one.

| Table | Identity | Responsibility |
|---|---|---|
| subscriptions | UUID; unique canonical URL | Explicitly selected sources, title, last success/error |
| items | source UUID + external ID; unique `(source_id, external_id)` | Normalized title, URL, publication/first-fetch timestamps |
| item_states | item ID | Device-local explicit read/unread state |
| connected_accounts | account ID | Future account display/status and credential reference, never secrets |

Unsubscribing cascades deletion of that source's items and read state. This is device-local only and does not unfollow anything on an upstream platform. Duplicate subscription URLs return the existing source. Fragments are removed; other URL semantics are preserved.

Content upserts update title, URL and publication timestamp while retaining ID, first-fetch time and read state. Transaction rollback prevents partial source batches. Failed refresh does not advance the last-success timestamp. Undated entries use their first-fetch timestamp for stable ordering. Times used for sorting are normalized to UTC with millisecond precision.

No encryption of cached metadata is claimed. Data is stored in the OS application-data directory under `app.mediaone.client`; deleting the app may remove it depending on OS behavior. Export/import, retention controls, saved items and cross-device synchronization remain future work.
