# Aissistant

[English](README.md) | [简体中文](README.zh-CN.md)

**Aissistant** 是一个轻量级 Windows 桌面 AI 助手，使用 Rust 和 egui 构建。它可以常驻系统托盘，通过全局快捷键快速唤起，并支持 DeepSeek、OpenAI、OpenRouter 等 OpenAI 兼容 API。

> 当前状态：早期可用版本，适合个人日常使用，也适合作为轻量桌面 AI 助手继续扩展。

## 功能

- 原生 Windows 桌面应用，体积小，启动快
- 系统托盘常驻，支持关闭按钮隐藏到托盘
- 全局快捷键显示或隐藏窗口，默认 `Ctrl+Space`
- 限制单实例运行，重复启动会唤醒已有窗口
- OpenAI 兼容 Chat Completions API
- API Key 使用 Windows DPAPI 加密保存
- 请求完成后事件唤醒 UI，后台隐藏时减少无意义轮询
- 可取消当前请求，并忽略旧请求结果
- 回答支持基础 Markdown 风格、代码块复制、整段复制和清空
- 自定义窗口、托盘和 exe 图标

## 技术栈

- Rust
- egui / eframe
- reqwest
- windows-sys
- OpenAI-compatible Chat Completions API

## 环境要求

- Windows 10 或更高版本
- Rust stable 工具链
- 可选：MinGW `windres`，用于嵌入 exe 图标

## 快速开始

```powershell
git clone https://github.com/your-name/aissistant.git
cd aissistant
cargo run
```

构建 release：

```powershell
.\scripts\build_release.ps1
```

如果 `windres` 不在 PATH 中，可以先设置：

```powershell
$env:MINGW_BIN = "D:\path\to\mingw64\bin"
.\scripts\build_release.ps1
```

输出文件：

```text
target\release\aissistant.exe
```

## 配置

打开应用后进入 `Settings` 页面：

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

## 隐私说明

配置保存在本机。API Key 在 Windows 上使用当前用户的 DPAPI 加密，旧版本明文配置会在下次保存时迁移为加密字段。你的提问只会发送到你配置的 API 服务商。

## 项目结构

```text
.
|-- src
|   |-- api.rs        # Chat API 请求逻辑
|   |-- config.rs     # 用户配置读写
|   |-- icon.rs       # 窗口、托盘和 exe 图标处理
|   |-- main.rs       # egui 应用界面
|   |-- platform.rs   # Windows 托盘、全局快捷键、单实例
|   `-- secret.rs     # API Key 加密/解密
|-- scripts
|   `-- build_release.ps1
|-- build.rs          # Windows exe 图标嵌入
|-- Cargo.toml
`-- README.md
```

## 路线图

- 流式输出
- 对话历史和多会话管理
- 服务商预设
- 配置导入导出
- 开机自启动
- 安装包和 GitHub Actions 自动发布
