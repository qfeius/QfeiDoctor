# QfeiDoctor - 网络诊断桌面工具
Tauri v2 + React/TypeScript + Rust

## 目录结构

```
src/                    # 前端 React/TypeScript
  components/           # UI 组件
  hooks/                # React hooks
  styles/               # CSS 样式
  types/                # TypeScript 类型定义
src-tauri/              # Rust 后端
  src/
    diagnostics/        # 诊断引擎核心（DNS/TCP/TLS/HTTP/System）
    lib.rs              # 库入口，Tauri 命令注册
    main.rs             # 应用入口
  capabilities/         # Tauri v2 安全能力声明
docs/                   # 架构文档
```

## 开发规范

### 代码修改后必须执行

```bash
make format-fix   # 自动修复格式（cargo fmt + prettier）
make format       # 检查格式是否通过（CI 同款）
```

**铁律：任何代码修改后，提交前必须执行 `make format` 确认通过。格式检查失败不允许提交。**

### 常用命令

| 命令 | 用途 |
|---|---|
| `make dev` | 启动开发服务器 |
| `make build` | 构建项目 |
| `make test` | 运行全部测试（Rust + 前端） |
| `make lint` | 代码检查（clippy + eslint） |
| `make format` | 格式检查（cargo fmt --check + prettier --check） |
| `make format-fix` | 自动修复格式 |
| `make preview` | 构建 debug .app 并打开 |
| `make all` | format + lint + test + build |

### 技术栈

- **前端**: React 18, TypeScript, Vite, Prettier
- **后端**: Rust, Tauri v2
- **格式化**: `cargo fmt`（Rust）, `prettier`（TS/CSS）
- **检查**: `cargo clippy`, `eslint`
- **测试**: `cargo test`, `vitest`

### 平台相关

- 构建目标: macOS + Windows（无 Linux）
- Windows 安装器: NSIS（`installMode: perMachine`，需 UAC 提权）
- 平台条件编译: `#[cfg(target_os = "...")]` 用于系统特定逻辑
