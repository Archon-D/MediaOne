# Connector contract

`PlatformConnector: Send + Sync` exposes a platform ID and an asynchronous `fetch(source)` returning a normalized title and batch of items. Connectors do not mutate SQLite or read state. Ingestion verifies every item's source ID before committing the batch. Implementations must use stable external identities and validated HTTP(S) links.

| Platform | Implementation | Capabilities today |
|---|---|---|
| RSS / Atom | `RssConnector` | Manual source, public document fetch |
| YouTube | Catalog placeholder | None |
| Bluesky | Catalog placeholder | None |
| X | Catalog placeholder | None |
| Rednote | Catalog placeholder | None |

The catalog does not pretend unsupported adapters work; account UI has no enabled login action. Add a real adapter and capability declaration together. Once a second adapter is implemented, replace RSS-only ingestion dispatch with a registry and migrate the RSS-only subscription constraint. Add authenticated source/account association only after secure vault support exists.

RSS uses `feed-rs` for RSS/Atom parsing and stable IDs, resolves relative article links against the feed URL, prefers alternate links and discards non-HTTP(S) links. Titles only are retained for this first slice. Feed errors leave cached content intact. A successful report's item count is the number returned by the document, **not newly inserted items**.

Future contract additions: auth context, cancellation, conditional requests, backoff, import and paginated/incremental cursors. Capability flags intentionally report these features as unavailable today. No recommendation endpoint should populate a normalized source batch.

Fixtures exercise RSS and Atom offline. HTTP integration tests use loopback responses instead of depending on public feed availability.
