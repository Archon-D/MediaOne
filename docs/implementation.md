# Implementation status

## Delivered by this skeleton

A native local RSS/Atom reader vertical slice, shared Rust/mobile entry point, responsive React UI, persistent SQLite store, migration, explicit source eligibility, stable chronological ordering, deduplication, read/unread operations, per-source refresh status, connector catalog and session/vault abstractions.

## Intentionally deferred

- OAuth, cookies, login windows and secure credential-vault implementations.
- Actual YouTube, Bluesky, X and Rednote adapters.
- Guaranteed background refresh and device-to-device synchronization.
- Full article bodies, media playback, collections, search, saved state and exports.
- Incremental/conditional fetch, retry/backoff, cursor pagination and retention.
- Checked-in mobile SDK projects, signing and store packages.

## Validation

Frontend tests mock the IPC boundary and verify add-source, read/unread, source failure reporting and text escaping. Rust tests cover RSS/Atom parsing, URL validation, stable IDs, HTTP ingestion, migrations, deduplication, sorting, filtering, read preservation, unsubscribe cascading, rollback and reopen persistence. CI runs frontend build/tests plus native Rust tests/check on macOS.

Mobile target compilation and on-device runtime tests remain necessary before calling the mobile client verified. No desktop/mobile binary release is created by this draft PR.

## Verified baseline

2026-10-18: local TypeScript production build and 3 React interaction tests passed. GitHub Actions on macOS passed 5 Rust tests and native `cargo check`; the Linux frontend job also passed. The successful run's generated Cargo.lock and rustfmt output are checked in so the native dependency set is reproducible. Mobile on-device execution and Windows/Linux native builds remain unverified.
