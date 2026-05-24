# Aissistant

[English](README.md) | [简体中文](README.zh-CN.md)

**Aissistant** is a lightweight Windows desktop AI assistant built with Rust and egui. It stays quietly in the system tray, can be opened with a global hotkey, and works with OpenAI-compatible APIs such as DeepSeek, OpenAI, OpenRouter, and more.

> Status: early but usable. The project is suitable for personal daily use and further development.

## Preview

Screenshots will be added later.

## Features

- Lightweight native Windows desktop app
- Small chat window with system tray support
- Global hotkey to show or hide the window
- Customizable hotkey, default `Ctrl+Space`
- OpenAI-compatible chat API support
- Works with DeepSeek, OpenAI, OpenRouter, and similar providers
- Separate settings page for API configuration
- Auto-focus input box when the window opens
- `Enter` to send, `Ctrl+Enter` for newline
- Close button can hide the app to system tray
- Tray left click to show or hide
- Tray right click menu with `Show / Hide` and `Exit`
- Rendered answer view with basic Markdown-like support
- Code block display with copy button
- Inline code and formula block styling
- Custom window, tray, and exe icon
- Release build optimized for small binary size

## Tech Stack

- Rust
- egui / eframe
- reqwest
- windows-sys
- OpenAI-compatible Chat Completions API

## Requirements

- Windows 10 or later
- Rust stable toolchain
- MinGW toolchain with `windres`, or another working Windows linker

## Quick Start

Clone the repository:

```powershell
git clone https://github.com/your-name/aissistant.git
cd aissistant
```

Run in development mode:

```powershell
cargo run
```

Build release exe:

```powershell
.\scripts\build_release.ps1
```

Output:

```text
target\release\aissistant.exe
```

## Configuration

Open the app and go to `Settings`.

Required fields:

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

## Supported Providers

Any provider compatible with the OpenAI Chat Completions API should work.

Known examples:

- DeepSeek
- OpenAI
- OpenRouter
- Local proxy services compatible with `/chat/completions`

## Project Structure

```text
.
├── src
│   ├── api.rs        # Chat API request logic
│   ├── config.rs     # User configuration load/save
│   ├── icon.rs       # Window, tray, and exe icon helpers
│   ├── main.rs       # egui application UI
│   └── platform.rs   # Windows tray and global hotkey integration
├── scripts
│   └── build_release.ps1
├── build.rs          # Windows exe icon embedding
├── Cargo.toml
└── README.md
```

## Roadmap

- Streaming responses
- Conversation history
- Multiple conversations
- Better Markdown rendering
- Better LaTeX / formula rendering
- Provider presets
- Import/export settings
- Auto start on boot
- Installer packaging
- GitHub Actions release builds

## Privacy

Aissistant stores configuration locally on your machine. Your prompts are sent only to the API provider you configure.

Do not publish your API key. If you open an issue, remove sensitive information from logs and screenshots.

## Contributing

Contributions are welcome. You can help by:

- Reporting bugs
- Suggesting features
- Improving UI/UX
- Adding provider presets
- Improving documentation
- Submitting pull requests

## License

License has not been selected yet.

Recommended options:

- MIT: simple and permissive
- Apache-2.0: permissive with explicit patent grant
- GPL-3.0: requires derivative works to stay open source
