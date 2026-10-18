# MediaOne

A local-first subscription reader built with **Tauri 2, React, TypeScript, Rust and SQLite**. Every device fetches its own content. No server, browser extension or desktop collection node.

## Implemented

- Add/remove RSS and Atom sources, fetch with a bounded Rust HTTP client.
- Local SQLite migrations, transactional deduplication and persistent independent read state.
- Newest-first reading room with source/unread filters; up to 200 newest articles per view.
- Manual refresh and five-minute refresh while the app is visible; individual source errors remain visible.
- Connector capability catalog and explicit placeholders for YouTube, Bluesky, X and Rednote.
- Account metadata schema and credential-vault/session interfaces. Login and secret storage are **not implemented**.

## Development

Install Node.js 22.12+, a current stable Rust toolchain, and [Tauri platform prerequisites](https://v2.tauri.app/start/prerequisites/).

```sh
npm ci
npm run tauri dev
```

`npm run dev` alone serves the UI for development, but cannot access local data; it displays a native-app requirement rather than fabricated feeds. RSS fetches run in Rust, so browser CORS does not govern ingestion.

```sh
npm test
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
```

For mobile, first install the respective platform SDKs and Rust targets, then generate the native project:

```sh
npm run tauri android init
npm run tauri android dev
# macOS + Xcode only:
npm run tauri ios init
npm run tauri ios dev
```

Generated SDK projects are ignored. Mobile builds use the same Rust commands, connector and device-local SQLite database; they do not communicate with a desktop. Android needs internet permission (Tauri's generated project includes it). HTTPS feeds work across targets; plain HTTP can be blocked by iOS/Android network policies. No unrestricted cleartext exception is included.

## Try the first workflow

1. Open **Subscriptions**, enter a real RSS/Atom URL, and select **Add source**.
2. Select **Refresh feeds**, then **Reading room**.
3. Mark articles read, filter unread, restart the app and verify state persists.
4. Disconnect networking: cached titles and read actions still work. Refresh failures appear in Subscriptions.

Article titles are rendered as text and originals open through HTTP(S) links; remote feed HTML/scripts are never injected. This skeleton caches metadata and titles, not full articles or media.

See [architecture](docs/architecture.md), [connector contract](docs/connectors.md), [data model](docs/data-model.md), and [implementation status](docs/implementation.md).
