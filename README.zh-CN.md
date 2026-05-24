# Aissistant

[English](README.md) | [简体中文](README.zh-CN.md)

**Aissistant** 是一个轻量级 Windows 桌面 AI 助手，使用 Rust 和 egui 构建。它可以常驻系统托盘，通过全局快捷键快速唤起，并支持 DeepSeek、OpenAI、OpenRouter 等 OpenAI 兼容 API。

> 当前状态：早期可用版本。适合个人日常使用，也适合作为轻量桌面 AI 助手继续扩展。

## 预览

截图后续补充。

## 功能

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

## 技术栈

- Rust
- egui / eframe
- reqwest
- windows-sys
- OpenAI 兼容 Chat Completions API

## 环境要求

- Windows 10 或更高版本
- Rust stable 工具链
- MinGW 工具链并包含 `windres`，或其他可用的 Windows 链接器

## 快速开始

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

## 配置

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

## 支持的服务商

理论上，任何兼容 OpenAI Chat Completions API 的服务都可以使用。

已知示例：

- DeepSeek
- OpenAI
- OpenRouter
- 兼容 `/chat/completions` 的本地代理服务

## 项目结构

```text
.
├── src
│   ├── api.rs        # Chat API 请求逻辑
│   ├── config.rs     # 用户配置读取和保存
│   ├── icon.rs       # 窗口、托盘和 exe 图标处理
│   ├── main.rs       # egui 应用界面
│   └── platform.rs   # Windows 托盘和全局快捷键集成
├── scripts
│   └── build_release.ps1
├── build.rs          # Windows exe 图标嵌入
├── Cargo.toml
└── README.md
```

## 路线图

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

## 隐私说明

Aissistant 的配置保存在本机。你的提问只会发送到你配置的 API 服务商。

请不要公开 API Key。提交 issue 时，请从日志和截图中移除敏感信息。

## 贡献

欢迎贡献。你可以通过以下方式参与：

- 反馈 bug
- 提出功能建议
- 改进 UI/UX
- 增加服务商预设
- 完善文档
- 提交 pull request

## 许可证

当前尚未选择许可证。

推荐选项：

- MIT：简单宽松
- Apache-2.0：宽松，并包含明确的专利授权条款
- GPL-3.0：要求衍生作品继续开源
