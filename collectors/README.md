# Collectors (Phase 1 MVP)

Phase 1 目标：三端都能生成 bundle，包含 `session_started`、`app_focus_changed`、`session_stopped`，并通过 verifier。

本仓库用统一的 Rust CLI 作为最小可运行 collector，分别在 macOS/Windows/Linux 目录下提供运行脚本。

通用步骤：

1. 启动 core server（生成 Evidence bundle）  
2. 发送 `app_focus_changed`  
3. 发送 `file_added`（录屏文件/占位文件）  
4. 发送 `shot_saved`（截图文件）  
5. 发送 `input_stats`（输入统计）  
6. 发送 `stop` 结束会话  

使用详情见各平台子目录。

默认输出目录：`~/Downloads`（可通过 `aegis-core-server` 的 `save_dir` 参数覆盖）。
