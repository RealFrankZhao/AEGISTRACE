# Evidence Bundle 规范（Phase 0-2）

目标：确保三端输出结构一致、可被独立验证，并具备防篡改能力。

## 目录结构（固定）

```
Evidence_YYYYMMDD_HHMMSS/
  session.json
  events.jsonl
  manifest.json
  files/
    screen.mp4 / screen.mkv / screen.mov   # 可选
    shots/                                  # 可选
```

## session.json

最小字段：

- `started_at`：UTC ISO8601
- `ended_at`：UTC ISO8601（可为空）
- `platform`：`macos` / `windows` / `linux`
- `app_version`：采集器版本
- `bundle_dir`：Evidence 目录名

## events.jsonl

每行一个事件（JSON object）。

必填字段（Phase 0）：

- `seq`：严格递增（从 1 开始）
- `ts`：UTC ISO8601
- `type`：事件类型（字符串）
- `payload`：JSON object

防篡改字段（Phase 2）：

- `prev_hash`：上一条事件的 `hash`（第一条为空字符串）
- `hash`：当前事件哈希

统一事件类型（跨平台对齐）：

- `session_started { save_dir, platform, app_version }`
- `session_stopped { reason }`
- `app_focus_changed { app_id, app_name, window_title? }`
- `file_added { rel_path, kind }`
- `shot_saved { rel_path }`
- `input_stats { interval_ms, key_count, backspace_count, paste_count, idle_bins... }`
- `net_domain { domain, app_id?, direction }`

## manifest.json

最小字段（Phase 0-2）：

- `schema_version`：固定为 `1`
- `events_hash`：`events.jsonl` 的 SHA-256
- `final_hash`：最后一条事件的 `hash`
- `files`：文件数组（可为空）

文件条目字段：

- `rel_path`：相对路径
- `hash`：文件 SHA-256

## Phase 0 验收（Verifier v0）

- `session.json`、`events.jsonl`、`manifest.json` 必须存在
- `events.jsonl` 中 `seq` 连续、从 1 开始

## Phase 2 验收（Verifier v1）

- 校验 `prev_hash/hash` 哈希链
- 校验 `events_hash` 与实际文件一致
- 校验 `final_hash` 与最后事件一致
- 校验 `manifest.files` 中列出的文件存在
