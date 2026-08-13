# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A desktop application built with Tauri 2.0, utilizing a Rust backend and a React + TypeScript frontend. The system implements a robust "Smart File Search" capability featuring background indexing of the file system, metadata extraction, and high-performance querying.

## Development Commands

### Frontend (Vite/React)

- `npm run dev`: Starts the Vite development server for the frontend.
- `npm run build`: Compiles frontend assets (runs `tsc` before building).
- `npm run preview`: Previews the built production application.
- `npm run format`: Runs Prettier to ensure consistent code formatting across the project.

### Backend (Rust/Tauri)

- `npm run tauri`: Entry point for Tauri commands (e.g., `npm run tauri dev` or `npm run tauri build`).
- `cargo test`: Run Rust backend unit and integration tests from the `src-tauri` directory.
- `cargo check`: Verify Rust compilation without building to speed up local development loops.

## Architecture Overview

### Backend Construction (`src-tauri/`)

The backend is structured into three primary layers:

1.  **API Layer (`commands/`)**: Exposes high-level functionality to the frontend via Tauri's command system. These use **specta** to automatically generate type-safe TypeScript bindings, ensuring a seamless bridge between Rust and TypeScript.
2.  **Service Layer (`services/`)**: Contains core business logic.
    - `indexer`: Implements a multi-threaded producer-consumer model (using `tokio::mpsc` and `WalkBuilder`) to crawl the filesystem and process files asynchronously in the background.
    - `database`: Manages the SQLite connection and low-level data interactions.
3.  **State Management**: The backend maintains an `AppState` (accessible via `tauri::State`) which holds shared resources like the database connection, configuration manager, and blacklist filters.

### Frontend Construction (`src/`)

1.  **UI Component Layer**: Built with React and styled using Tailwind CSS with a focus on modular components.
2.  **State Management**: Uses **Zustand** for managing global application state (e.g., search results, configuration status, and UI interactions).
3.  **Bridge Layer (`src/bindings/`)**: Integrates the frontend with the backend using generated bindings to ensure type safety across the bridge.

### Data & Search Engine

- **Indexing Pipeline**: Files are identified via **blake3** hashing for unique identification; metadata (modification time, size, MIME types) is extracted during the crawl phase.
- **Storage Layer**: Employs a combination of `rusqlite` and `sqlite-vec` to support both relational data management and high-performance search over indexed content.

## Key Logic Paths

- **File Discovery**: Managed in `src-tauri/src/services/indexer/`. It utilizes the `ignore` crate to respect system conventions (like `.gitignore` patterns) while filtering out items based on user-defined blacklists.
- **Configuration**: Handles persistent application settings via `json5` for flexible parsing of configuration files.
- **Search Flow**: Frontend $\rightarrow$ Tauri Commands $\rightarrow$ Rust Services $\rightarrow$ SQLite/Sqlite-vec Query $\rightarrow$ Result Mapping $\rightarrow$ Frontend State Update.

## Technical Stack

- **Frontend**: React, TypeScript, Tailwind CSS, Vite, Zustand.
- **Backend**: Rust, Tauri 2.0, Tokio (Async), Rusqlite, Blake3.
- **Tooling**: Specta (Type Generation), Prettier, Vite.
