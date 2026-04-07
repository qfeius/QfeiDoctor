# AI Changes

## 2026-04-07
- 变更摘要：修复新安装包启动即崩的问题。
- 涉及文件/模块：`src-tauri/tauri.conf.json`、`src/components/__tests__/tauriShellScope.test.ts`。
- 关键逻辑/决策：定位到 release 安装包启动后以 Rust `101` 退出，`tauri dev` 抓到 panic 为 `plugins.shell.open` 配置结构错误；将 `shell.open` 从数组改为 `tauri-plugin-shell` 支持的单个正则字符串。
- 验证：单测先失败后通过，`npm test` 通过，`npm run build` 通过，重新生成 NSIS 安装包后 `target/release/QfeiDoctor.exe` 启动 5 秒仍存活。

## 2026-04-07
- 变更摘要：补齐本机 Windows 原生构建链并产出 NSIS 安装包。
- 涉及文件/模块：`src/components/__tests__/tauriShellScope.test.ts`、Windows Build Tools / Tauri 打包链路。
- 关键逻辑/决策：将回归测试改为通过 Vite `?raw` 读取 `tauri.conf.json`，避免 Node 类型依赖阻塞 `tsc`；构建时导入本机 `Visual Studio 18 BuildTools` 的 `vcvars64` 环境，确保 `link.exe` 可被 Cargo 使用。
- 验证：`npm test` 通过，`npm run build` 通过，`npm run tauri build -- --bundles nsis` 成功生成 `src-tauri/target/release/bundle/nsis/QfeiDoctor_0.1.0_x64-setup.exe`。

## 2026-04-07
- 变更摘要：修复 Windows 下“打开代理设置”无响应的问题。
- 涉及文件/模块：`src-tauri/tauri.conf.json`、`src/components/__tests__/tauriShellScope.test.ts`。
- 关键逻辑/决策：确认 `@tauri-apps/plugin-shell` 默认只允许 `mailto/tel/http(s)`，补充 `plugins.shell.open` 显式放行 `ms-settings:` 与 macOS 系统设置 URI。
- 验证：新增回归测试先失败后通过，并执行 `npm test` 全量通过；`npm run tauri info` 可正常解析当前 Tauri 配置。
