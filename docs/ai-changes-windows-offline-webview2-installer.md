# Windows 离线 WebView2 安装变更

## 2026-09-11

- 变更摘要：Windows 安装包改为内置 WebView2 离线安装器，避免安装阶段因客户网络、代理或 Microsoft 下载地址不可用而触发 MSI 1722 错误。
- 涉及文件/模块：`src-tauri/tauri.conf.json`、`src/components/__tests__/tauriWindowsInstaller.test.ts`。
- 关键逻辑/决策：显式将 `bundle.windows.webviewInstallMode.type` 设置为 `offlineInstaller`；保留 NSIS 的 `perMachine` 安装模式和现有 MSI/NSIS 双产物策略。
- 影响与权衡：Windows 安装不再依赖在线下载 WebView2，但安装包体积预计增加约 127 MB。
- 验证策略：先增加配置回归测试并确认其在旧配置下失败，再修改配置使测试通过；同时运行前端全量测试、格式检查、Lint、构建及 Tauri 配置解析检查。
