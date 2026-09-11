# Release 版本标识优化

## 2026-09-11

- 变更摘要：发布工作流以 Git 标签为唯一版本源，使 Release 标签、安装包文件名和应用内部版本保持一致。
- 涉及文件/模块：`.github/workflows/release.yml`、`scripts/prepare-release-config.mjs`、`scripts/prepare-release-config.test.mjs`、`.gitignore`。
- 关键逻辑/决策：发布前严格校验 `v主版本.次版本.修订号` 格式，将去除 `v` 前缀后的版本写入临时 Tauri 覆盖配置，再通过 `tauri-action` 的 `--config` 参数参与 macOS 和 Windows 打包。
- 安全与兼容性：标签通过环境变量传递，不直接拼接进 Shell 命令；使用 Node.js 脚本保证 macOS 与 Windows Runner 行为一致；非法标签会在打包前明确失败。
- 验证：标签解析与工作流接线测试均先失败后通过；前端 13 项测试、Rust 20 项测试、格式检查、Lint 和前端构建通过；使用测试标签 `v2.3.4` 完成本地 Tauri 打包，产物的 `CFBundleShortVersionString` 与 `CFBundleVersion` 均为 `2.3.4`。
