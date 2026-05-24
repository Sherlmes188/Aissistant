# Aissistant

[English](README.md) | [Simplified Chinese](README.zh-CN.md)

**Aissistant** is a lightweight Windows desktop AI assistant built with Rust and egui. It lives in the system tray, opens with a global hotkey, and works with OpenAI-compatible APIs such as DeepSeek, OpenAI, OpenRouter, and local proxy services.

> Status: early but usable. The project is suitable for daily personal use and further development.

## Features

- Lightweight native Windows desktop app
- System tray support with close-to-tray behavior
- Global hotkey to show or hide the window, default `Ctrl+Space`
- Single-instance mode; launching again wakes the existing window
- OpenAI-compatible Chat Completions API support
- API key protected with Windows DPAPI
- Request completion wakes the UI directly, reducing background polling
- Cancel current request and ignore stale responses
- Basic Markdown-like answer rendering
- Code block copy, full-answer copy, and clear controls
- Embedded window, tray, and exe icon

## Tech Stack

- Rust
- egui / eframe
- reqwest
- windows-sys
- OpenAI-compatible Chat Completions API

## Requirements

- Windows 10 or later
- Rust stable toolchain
- Optional: MinGW `windres` for embedding the exe icon

## Quick Start

```powershell
git clone https://github.com/your-name/aissistant.git
cd aissistant
cargo run
```

Build release exe:

```powershell
.\scripts\build_release.ps1
```

If `windres` is not in PATH, set `MINGW_BIN` first:

```powershell
$env:MINGW_BIN = "D:\path\to\mingw64\bin"
.\scripts\build_release.ps1
```

Output:

```text
target\release\aissistant.exe
```

## Configuration

Open the app and go to `Settings`.

- `Base URL`: API endpoint, for example `https://api.deepseek.com/v1`
- `API Key`: your provider API key
- `Model`: model name, for example `deepseek-chat`
- `System Prompt`: assistant behavior prompt
- `Global Hotkey`: show or hide shortcut

Hotkey examples:

```text
Ctrl+Space
Alt+Space
Ctrl+Alt+A
Ctrl+Shift+K
```

## Privacy

Configuration is stored locally. On Windows, the API key is encrypted with the current user's DPAPI profile. Older plaintext configs are loaded for compatibility and migrated to the encrypted field on the next save. Your questions are sent only to the API provider configured by you.

## Project Structure

```text
.
|-- src
|   |-- api.rs        # Chat API request logic
|   |-- config.rs     # User configuration load/save
|   |-- icon.rs       # Window, tray, and exe icon helpers
|   |-- main.rs       # egui application UI
|   |-- platform.rs   # Windows tray, global hotkey, single-instance integration
|   `-- secret.rs     # API key protection
|-- scripts
|   `-- build_release.ps1
|-- build.rs          # Windows exe icon embedding
|-- Cargo.toml
`-- README.md
```

## Roadmap

- Streaming output
- Conversation history and multi-session management
- Provider presets
- Configuration import/export
- Start on login
- Installer and GitHub Actions release builds
