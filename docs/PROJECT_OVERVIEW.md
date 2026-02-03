# AEGISTRACE 当前成果总览（供 Windows/Linux 后续落地）

本文件总结当前仓库已实现的功能、技术栈、文件结构、数据流、事件模型与录屏分段规则，作为 Windows/Linux 端对齐实现的参考。

## 1) 当前仓库目录结构

```
AEGISTRACE/
  apps/
    aegis-tauri/                 # GUI 壳（Tauri v2）
      ui/                        # 纯静态 UI
      src-tauri/                 # Tauri 后端（Rust）
  crates/
    aegis-core/                  # 证据包写入（事件、哈希链、manifest）
    aegis-core-server/           # TCP 服务器（Collector IPC）
    aegis-collector-cli/         # CLI 发送事件（调试/最小采集）
    aegis-verifier/              # 验证器（结构+hash+文件）
  collectors/
    macos/
      native_recorder/           # 原生录屏（Swift/AVFoundation）
      run_demo.sh                # macOS demo 脚本
    windows/                     # 预留
    linux/                       # 预留
  spec/
    evidence_bundle.md           # 证据包规范
  scripts/
    build_macos.sh               # macOS 构建产物
    build_linux.sh               # Linux 构建产物
    build_windows.ps1            # Windows 构建产物
    macos_app_bundle.sh          # macOS .app 打包
  docs/
    PROJECT_OVERVIEW.md          # 本文档
```

## 2) 技术栈与职责划分

- Rust `aegis-core`: 写入证据包、事件哈希链、manifest 文件清单与哈希。
- Rust `aegis-core-server`: TCP 监听 IPC，将事件写入 `aegis-core`，并复制文件到 bundle。
- Rust `aegis-collector-cli`: 调试/最小采集 CLI，发 `focus/file/shot/input/stop`。
- Rust `aegis-verifier`: 验证结构、哈希链、manifest 与文件存在性。
- macOS 原生录屏：Swift + AVFoundation（H.265/HEVC，720p@30fps，≈2Mbps）。
- GUI：Tauri v2 + 静态 HTML（Start/Stop 控制）。

## 3) 核心数据结构与证据包规范

详见 `spec/evidence_bundle.md`。关键点：

- 证据目录固定：

```
Evidence_YYYYMMDD_HHMMSS/
  session.json
  events.jsonl
  manifest.json
  files/
    screen_*.mov                # 录屏分段
    shots/000001.jpg            # 截图
```

- `events.jsonl` 每行一个事件，字段：
  - `seq` 递增
  - `ts` UTC 时间
  - `type` 事件类型
  - `payload` JSON
  - `prev_hash` / `hash`（防篡改）
- `manifest.json`：`events_hash` + `final_hash` + 文件清单哈希。

## 4) 事件类型（跨平台统一）

- `session_started { save_dir, platform, app_version }`
- `session_stopped { reason }`
- `app_focus_changed { app_id, app_name, window_title? }`
- `file_added { rel_path, kind }`
- `shot_saved { rel_path }`
- `input_stats { interval_ms, key_count, backspace_count, paste_count }`

## 5) IPC 协议（当前实现）

- 传输：TCP `127.0.0.1:7878`
- 格式：JSON 每行一条消息
- 示例：
```
{ "type": "app_focus_changed", "payload": { "app_id": "...", "app_name": "..." } }
```

## 6) 录屏实现与分段规则

### 6.1 macOS 原生录屏（Swift）

文件：`collectors/macos/native_recorder/Sources/main.swift`

核心参数：
- 编码：HEVC（H.265）
- 分辨率：按显示器缩放到 1280x720（保持比例）
- 帧率：30fps
- 码率：约 2Mbps
- 输出格式：`.mov`

### 6.2 GUI 端录屏分段

文件：`apps/aegis-tauri/src-tauri/src/main.rs`

机制：
- Start：启动 core-server + 启动录屏循环线程
- Stop：停止当前段、等待写入、停止线程、写入 `session_stopped`
- 每 10 分钟自动切段（600 秒）
- 文件名规则：
```
files/screen_<段号>_<timestamp>.mov
```

注意：分段逻辑在 GUI 端完成（Windows/Linux 后续可照搬或下沉到平台采集器）。

## 7) 组件关系与数据流（原理图）

```
┌──────────────┐         JSON/TCP         ┌──────────────────┐
│ GUI (Tauri)  │ ───────────────────────▶ │ aegis-core-server│
│ Start/Stop   │                          │ (TCP 127.0.0.1)  │
└──────┬───────┘                          └───────┬──────────┘
       │                                           │
       │ spawn/stop                                │ write events/files
       ▼                                           ▼
┌──────────────────┐                     ┌───────────────────────┐
│ native recorder  │                     │ aegis-core (writer)   │
│ (Swift/AVF)       │                     │ events + hash chain    │
└──────┬───────────┘                     └─────────┬─────────────┘
       │ copies segment files                         │ writes manifest
       ▼                                               ▼
┌────────────────────────────────────────────────────────────────┐
│ Evidence_YYYYMMDD_HHMMSS/                                      │
│  session.json / events.jsonl / manifest.json / files/*.mov     │
└────────────────────────────────────────────────────────────────┘

Verifier: aegis-verifier verify <bundle_path>
```

## 8) Windows/Linux 后续落地建议

### 8.1 复用策略

- `aegis-core`、`aegis-core-server`、`aegis-verifier` 保持不变。
- IPC 协议保持一致，`aegis-collector-cli` 作为调试基准。
- 平台采集器实现替换为：
  - Windows：.NET/C++ + Native screen capture + TCP JSON
  - Linux：Rust/C + Pipewire/FFmpeg + TCP JSON

### 8.2 录屏分段实现建议

保持 GUI 侧分段逻辑：
- Start 时启动分段线程
- 每 600 秒录制一段
- Stop 时写完最后一段并退出

或将分段逻辑内聚到平台采集器：
- 采集器直接按段输出文件并发送 `file_added` 消息

### 8.3 需要对齐的关键行为

- 统一文件命名与 `file_added` 事件格式
- 所有文件最终都必须落入 bundle `files/` 并参与 manifest hash
- 录屏输出格式建议 `.mov`（mac）/ `.mp4`（win/linux）均可，但需一致记录 `rel_path`

## 9) 当前功能验收清单

- GUI Start/Stop 可反复执行
- 证据包结构完整
- `events.jsonl` hash chain 校验通过
- `manifest.json` 文件 hash 一致
- `files/screen_<段号>_<timestamp>.mov` 按 10 分钟分段输出

## 10) 参考路径

- Core writer: `crates/aegis-core/src/lib.rs`
- TCP server: `crates/aegis-core-server/src/main.rs`
- Verifier: `crates/aegis-verifier/src/main.rs`
- Collector CLI: `crates/aegis-collector-cli/src/main.rs`
- macOS recorder: `collectors/macos/native_recorder/Sources/main.swift`
- Tauri backend: `apps/aegis-tauri/src-tauri/src/main.rs`
- Tauri UI: `apps/aegis-tauri/ui/index.html`
