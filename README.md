# QfeiDoctor

智书诊断助手，售后网络排查工具。

客户下载后输入 URL 或域名，自动诊断 DNS、TCP、TLS、HTTP、系统代理等问题，生成结构化 JSON 报告，复制发给售后工程师。

## Prerequisites

- [Node.js](https://nodejs.org/) >= 22
- [Rust](https://rustup.rs/) (stable)
- 系统依赖（仅 Linux）：
  ```bash
  sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
  ```

## Quick Start

```bash
# 安装依赖
npm ci

# 开发模式（启动 Tauri + Vite dev server）
make run
```

应用会自动打开一个桌面窗口，输入域名或 URL 即可开始诊断。

## Build

```bash
# 构建可执行文件（产物在 src-tauri/target/release/bundle/）
npm run tauri build
```

## Development Commands

```bash
make run           # 启动本地开发环境（Tauri + Vite）
make format        # 检查格式（CI 用）
make format-fix    # 自动格式化 Rust + TypeScript
make lint          # Clippy + ESLint
make test          # Rust 单元测试 + Vitest 前端测试
make build         # 编译 Rust + Vite 前端（不打包安装包）
```

## Project Structure

```
src/                    # React 前端
  components/           # UI 组件
  hooks/                # React hooks（useDiagnostic）
  types/                # TypeScript 类型定义
src-tauri/              # Rust 后端（Tauri）
  src/diagnostics/      # 诊断引擎
    dns.rs              # DNS 解析诊断
    tcp.rs              # TCP 连接诊断
    tls.rs              # TLS/SSL 证书诊断
    http.rs             # HTTP 请求诊断
    system.rs           # 系统代理 / 时钟偏差 / hosts 检测
    mod.rs              # 编排器 + recommended_actions 生成
    result.rs           # 结果类型定义（对齐 JSON schema）
  tests/                # 集成测试（smoke/acceptance）
docs/
  diagnostic-result.schema.json  # 诊断结果 JSON schema v0（source of truth）
```

## Docs

- `docs/PRD.md` — product scope and user flow
- `docs/ARCHITECTURE.md` — implementation constraints and module boundaries
- `docs/diagnostic-result.schema.json` — canonical diagnostic result contract for Rust / React / SQE
