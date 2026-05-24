# Aissistant

**Aissistant** is a lightweight Windows desktop AI assistant built with Rust and egui. It stays quietly in the system tray, can be opened with a global hotkey, and works with OpenAI-compatible APIs such as DeepSeek, OpenAI, OpenRouter, and more.

**Aissistant** 是一个轻量级 Windows 桌面 AI 助手，使用 Rust 和 egui 构建。它可以常驻系统托盘，通过全局快捷键快速唤起，并支持 DeepSeek、OpenAI、OpenRouter 等 OpenAI 兼容 API。

> Status: early but usable. The project is suitable for personal daily use and further development.
>
> 当前状态：早期可用版本。适合个人日常使用，也适合作为轻量桌面 AI 助手继续扩展。

## Preview / 预览

Screenshots will be added later.

截图后续补充。

## Features / 功能

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

中文：

- 轻量级 Windows 原生桌面应用
- 小窗口聊天界面，支持系统托盘
- 全局快捷键显示或隐藏窗口
- 快捷键可自定义，默认 `Ctrl+Space`
- 支持 OpenAI 兼容聊天 API
- 可接入 DeepSeek、OpenAI、OpenRouter 等服务
- API 配置独立放在设置页
- 窗口打开后自动聚焦输入框
- `Enter` 发送，`Ctrl+Enter` 换行
- 关闭按钮可隐藏到系统托盘
- 托盘左键显示或隐藏窗口
- 托盘右键菜单支持 `Show / Hide` 和 `Exit`
- 回答区域支持基础 Markdown 风格渲染
- 代码块展示和复制按钮
- 行内代码与公式块样式
- 自定义窗口、托盘和 exe 图标
- Release 构建针对小体积优化

## Tech Stack / 技术栈

- Rust
- egui / eframe
- reqwest
- windows-sys
- OpenAI-compatible Chat Completions API

## Requirements / 环境要求

- Windows 10 or later
- Rust stable toolchain
- MinGW toolchain with `windres`, or another working Windows linker

中文：

- Windows 10 或更高版本
- Rust stable 工具链
- MinGW 工具链并包含 `windres`，或其他可用的 Windows 链接器

## Quick Start / 快速开始

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

中文：

克隆项目：

```powershell
git clone https://github.com/your-name/aissistant.git
cd aissistant
```

开发模式运行：

```powershell
cargo run
```

编译 release 版本：

```powershell
.\scripts\build_release.ps1
```

输出文件：

```text
target\release\aissistant.exe
```

## Configuration / 配置

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

中文：

打开应用后进入 `Settings` 页面。

需要配置：

- `Base URL`：API 地址，例如 `https://api.deepseek.com/v1`
- `API Key`：服务商 API Key
- `Model`：模型名称，例如 `deepseek-chat`
- `System Prompt`：助手行为提示词
- `Global Hotkey`：显示或隐藏窗口的快捷键

快捷键示例：

```text
Ctrl+Space
Alt+Space
Ctrl+Alt+A
Ctrl+Shift+K
```

## Supported Providers / 支持的服务商

Any provider compatible with the OpenAI Chat Completions API should work.

Known examples:

- DeepSeek
- OpenAI
- OpenRouter
- Local proxy services compatible with `/chat/completions`

中文：

理论上，任何兼容 OpenAI Chat Completions API 的服务都可以使用。

已知示例：

- DeepSeek
- OpenAI
- OpenRouter
- 兼容 `/chat/completions` 的本地代理服务

## Project Structure / 项目结构

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

## Roadmap / 路线图

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

中文：

- 流式输出
- 对话历史
- 多会话管理
- 更完整的 Markdown 渲染
- 更好的 LaTeX / 公式渲染
- 常用服务商预设
- 配置导入导出
- 开机自启动
- 安装包打包
- GitHub Actions 自动构建发布版本

## Privacy / 隐私说明

Aissistant stores configuration locally on your machine. Your prompts are sent only to the API provider you configure.

Do not publish your API key. If you open an issue, remove sensitive information from logs and screenshots.

中文：

Aissistant 的配置保存在本机。你的提问只会发送到你配置的 API 服务商。

请不要公开 API Key。提交 issue 时，请从日志和截图中移除敏感信息。

## Contributing / 贡献

Contributions are welcome. You can help by:

- Reporting bugs
- Suggesting features
- Improving UI/UX
- Adding provider presets
- Improving documentation
- Submitting pull requests

中文：

欢迎贡献。你可以通过以下方式参与：

- 反馈 bug
- 提出功能建议
- 改进 UI/UX
- 增加服务商预设
- 完善文档
- 提交 pull request

## License / 许可证

License has not been selected yet.

Recommended options:

- MIT: simple and permissive
- Apache-2.0: permissive with explicit patent grant
- GPL-3.0: requires derivative works to stay open source

中文：

当前尚未选择许可证。

推荐选项：

- MIT：简单宽松
- Apache-2.0：宽松，并包含明确的专利授权条款
- GPL-3.0：要求衍生作品继续开源
