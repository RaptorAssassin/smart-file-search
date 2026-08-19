# Smart File Search

A privacy-first AI-powered desktop tool that lets you actually find what you search for.
Built with Tauri, Rust and Ollama.

![GitHub License](https://img.shields.io/github/license/RaptorAssassin/smart-file-search?label=License)
![App Version](https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fraw.githubusercontent.com%2FRaptorAssassin%2Fsmart-file-search%2Fmain%2Fpackage.json&query=%24.version&label=Version)

## Description

Smart File Search is a Tauri app for Desktop that find finds by their content and meaning instead of only searching for the filename. It indexes your files using local LLMs and embedding models through Ollama to guarantee full privacy.

- **Search for keywords**: FTS5 is used to search not only metadata but also AI-generated keywords that sum up the content of your files.
- **Search by meaning**: An embedding model helps to search for the actual meaning of a file without needing the exact wording.

## Why I Built This

After using the search in the file explorer on Windows, I noticed that it struggled to find what I really searched for. I wanted to build an alternative to better search my files, so I came up with the idea to build this app. I quickly decided to keep it local due to token costs getting too high for users with many files, which has the positive side effect of the app respecting the users's privacy which is important for an app that scans user files.

## Tech Stack

![Tauri Icon](https://img.shields.io/badge/Tauri-black?style=for-the-badge&logo=tauri&labelColor=black&color=%2324C8D8)
![React Icon](https://img.shields.io/badge/React-black?style=for-the-badge&logo=react&labelColor=black&color=%2361DAFB)
![Rust Icon](https://img.shields.io/badge/Rust-black?style=for-the-badge&logo=rust&labelColor=black&color=%23D34516)
![SQLite Icon](https://img.shields.io/badge/SQLite-black?style=for-the-badge&logo=sqlite&labelColor=black&color=%23003B57)
![Ollama Icon](https://img.shields.io/badge/Ollama-black?style=for-the-badge&logo=ollama&labelColor=black&color=%23FFFFFF)

- **Tauri**: Tauri is perfect for this project, as it allows me to easily write frontend code in React which I'm already familiar with while learning Rust for the backend at the same time.
- **React**: The frontend is built on React, as the Tauri frontend is a simple WebView. For storing state globally, I use the _Zustand_ library.
- **Rust**: Rust handles the heavy file processing in the background on multpiple threads while keeping everything performant and safe.
- **SQLite**: All indexing data is stored in a local _SQLite_ database.
- **Ollama**: Ollama is the go-to for using local models, both LLMs and embedding models are on there and used in this project.

## Features

- **File Indexing**: On app startup, the app scans all user files (excluding some from the system) and runs them through Ollama models to generate keywords and labels and embeddings. Then it saves the data into the local database, across three tables: One for standard metadata (Path, File size etc), one FTS5 table that saves the file content if the app could extract it, and an embedding table that stores the generated vector data.
- **Intelligent Search**: Search for files by metadata, content and meaning.
- **Configure Custom Model**: Pull whatever Ollama model you like and configure the app settings to use this model for indexing.
- **Keyboard Shortcuts**: The app is fully accessible with keyboard shortcuts and easy navigation.
- **Customization**: Users can toggle between dark, light or system theme and can toggle the visibility of keyboard shortcut hints.

## Upcoming Features

There are several features in planning for updates:

- **Search Filters**: Filter by file extension, size and more.
- **Usage Stats**: See how many files are indexed and how many AI tokens the app used.
- **Choose a Custom Embedding Model**: At the moment, only the normal keyword/label generation AI model can be fully chosen, the embedding model is locked to `nomic-embed-text`. In the future users will be able to select a custom model for this.

## Installation

Download the installer for your OS from the latest [release](https://github.com/RaptorAssassin/smart-file-search/releases):

- **Windows** — run the `.exe` installer.
- **macOS** — open the `.dmg` and drag the app into Applications.
- **Linux** — install the `.deb` with `sudo apt install ./smart-file-search_*_amd64.deb`, or run the `.AppImage` (install `libfuse2` first on Ubuntu 22.04+).

If your OS warns you before running the app for the first time:

- **Windows**: Click _More info_ → _Run anyway_
- **macOS**: Right-click the app and click _Open_.

## Ollama setup

To use the full AI-enhanced search of the app, you need Ollama models installed and running.

1. Install Ollama from [ollama.com](https://ollama.com/) and start it.
2. Pull the embedding model(`nomic-embed-text`):

   ```bash
   ollama pull nomic-embed-text
   ```

3. Pull a model for keyword generation and OCR. It should support both Image and Text Input (Recommendation: `gemma3:4b`).

   ```bash
   ollama pull gemma3:4b
   ```

4. When not using `gemma3:4b`, open the Settings with the button on the bottom left, choose _Custom_ and enter the model name. You can also specify a custom API endpoint for the keyword generation, but be careful as the usage might get very high depending on the amount of files you have.

Search works without Ollama indexing and embedding search, but can only use its full strength when used with the Ollama models running.
