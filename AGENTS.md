<!-- BEGIN:nextjs-agent-rules -->

# This is NOT the Next.js you know

This version has breaking changes — APIs, conventions, and file structure may all differ from your training data. Read the relevant guide in `node_modules/next/dist/docs/` (resolved from this file's directory; in monorepos the `next` package may not be visible from the repo root) before writing any code. Heed deprecation notices.

This block is written and re-added by `next dev` — verify at `node_modules/next/dist/server/lib/generate-agent-files.js`. Removing it from a diff only re-creates the uncommitted change; committing it with your work keeps the tree clean.

<!-- END:nextjs-agent-rules -->

# AGENTS.md

Tauri 2.0 desktop app (Rust backend + React 19/TS/Vite frontend): background filesystem indexing, metadata extraction, SQLite (FTS5 + sqlite-vec) storage.

## Code comments

Do not add code comments unless they are genuinely necessary. When a change involves non-obvious reasoning, explain it in your chat output instead of writing a comment.

## Commands

- `npm run tauri dev` — full app: Vite (fixed port 1420) + debug Rust build
- `npm run tauri build` — production build
- `npm run dev` / `npm run build` (runs `tsc && vite build`) — frontend only
- `npm run format` — Prettier over `src/` (repo style: no semicolons, single quotes)
- `cargo check` / `cargo test` — from `src-tauri/`. Rust tests exist in `services/indexer/blacklist.rs`, `commands/config/models.rs`, `services/ai/client.rs`, and `services/search/` (fusion, filters, metadata engines).
- No frontend test or lint setup.

## Generated bindings — do not edit by hand

`src/bindings/bindings.ts` is auto-exported by tauri-specta whenever the Rust backend builds in **debug** (`npm run tauri dev`, or any `cargo build`/`cargo run` from `src-tauri`). Plain `npm run dev` does NOT regenerate it.

- Commands need `#[tauri::command]` + `#[specta::specta]` and registration in `collect_commands!` inside `specta_builder()` (`src-tauri/src/lib.rs`).
- DTOs need `#[derive(specta::Type)]` (see `src-tauri/src/commands/config/models.rs`).
- specta is pinned to RC versions (`specta = "=2.0.0-rc.25"`, `tauri-specta = "2.0.0-rc.25"`, `specta-typescript = "0.0.12"`) — don't bump casually.
- Builder calls `.dangerously_cast_bigints_to_number()`; `i64`/`u64` cross the bridge as `number`.

## Architecture

- `src-tauri/src/commands/` — Tauri commands (config, debug). `services/` — `database.rs` (SQLite init/schema), `indexer/` (`indexer.rs` producer, `processing.rs` consumer, `blacklist.rs`).
- `AppState` (db_path, config_manager, blacklist) and `DbState` (`Mutex<rusqlite::Connection>`) are created in `lib.rs` setup and accessed via `tauri::State`.
- DB at `<app_config_dir>/app.db`; migrations keyed off `PRAGMA user_version` (currently 1). Schema: `files`, `files_fts` (FTS5 + sync triggers), `files_vec` (sqlite-vec, `embedding float[768]`, not yet populated). sqlite-vec is loaded via `sqlite3_auto_extension` with an unsafe transmute in `database.rs`.
- Indexing starts automatically in app setup: multi-threaded `ignore::WalkBuilder` producer (includes hidden files, disables .gitignore) → `tokio::mpsc` channel (capacity 1000) → sequential `process_file` consumer inserting into SQLite. Root is `/`. Every file is fully blake3-hashed — expensive.
- Blacklist merges bundled resource `data/blacklist.json5` (json5 syntax) with user `indexing.*` config stored in `config.json` (plain JSON via serde_json, not json5). Path patterns compile to a `globset::GlobSet` (`blacklist.rs`).
- Search (`services/search/`) — pluggable engines, one per retrieval method, fused in the backend:
  - Contract: each engine implements `SearchEngine { fn kind() -> EngineKind; async fn search(conn, query, filters) -> Result<Vec<RankedFile>, String> }`. The Vec's order *is* the vote — RRF never sees raw scores. `EngineKind` = `Metadata | Fts | Vector`; `RankedFile` = `{ file_id }`.
  - Orchestrator (`mod.rs`) runs engines sequentially on the single `Mutex<Connection>` (rusqlite `Connection` is `!Sync`), skips + flags any engine that errors, fuses survivors with Reciprocal Rank Fusion (`score = Σ 1/(60+rank)`), then hydrates full rows for the top N.
  - `metadata.rs` — tiered `CASE` match on plain columns: exact filename > filename prefix > filename substring > any-field substring; tiebreak `modified_at DESC`. Pure SQL, always available (no AI dependency), so search never fully fails.
  - `fts.rs` — `files_fts` bm25 (`ORDER BY rank`), filtered by allowed file ids. Returns *successful empty* until the AI pipeline populates `content_text`/`ai_summary`/`ai_keywords` — that is not an error.
  - `vector.rs` — embeds the query via Ollama `/api/embed` (768-d), then vec0 KNN (`WHERE embedding MATCH ? AND k = ? ORDER BY distance`), restricted to allowed file ids. Ollama down ⇒ engine error ⇒ skipped + flagged `unavailable`; an empty `files_vec` is a successful empty, not an error.
  - Filters are backend-side: `SearchFilters` (extensions, min/max `file_size`, modified-date range) build one shared WHERE fragment over `files` applied by every engine *before* ranking; frontend chip toggles re-query. `search_filter_options` exposes distinct extensions/categories + size/date bounds so the UI can render chip choices.
  - `SearchResult` carries `file_name`, `file_path`, `extension`, `category`, `mime_type`, `file_size`, `created_at`, `modified_at`, fused `score` — the frontend derives icons/colors and removable filter chips from these, with no extra round trips.
  - Commands: `search_files(query, filters, limit)` → `SearchResponse { results, unavailable }`. DTOs need `#[derive(specta::Type)]`, and commands are registered in `collect_commands!` (`src-tauri/src/lib.rs`); bindings regenerate on debug builds.
  - Folders are NOT indexed (`indexer.rs` only emits `path.is_file()` entries). Adding folders later = `file_type` migration + indexer emitting dirs + folders voting from the metadata engine only.
  - Deferred: multi-term queries, per-engine weights, connection pooling for parallel engines.

## Gotchas

- Windows is the dev platform; inode/`MetadataExt` code is `#[cfg(unix)]`.
- Keep the `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` line in `main.rs` — removing it reopens a console window in release.
- Tailwind v4 via `@tailwindcss/vite` — no `tailwind.config.js`; theme tokens live in `src/App.css`. shadcn components in `src/components/ui`. `@/*` resolves to `src/`.
- Vite must keep strict port 1420 (Tauri expects it) and ignores `src-tauri/` for watching.
- `src-tauri/gen/` is Tauri-generated schemas; `src-tauri/target/` is build output.
