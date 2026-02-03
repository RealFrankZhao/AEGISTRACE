# macOS Collector (MVP)

最小可运行 collector：使用 Rust CLI + core server。

运行：

```
./run_demo.sh
```

说明：
- 默认优先使用原生录屏（Swift + AVFoundation）生成 `screen.mp4`
- 如果没有 `swift` 或构建失败，会退回到 `ffmpeg`（黑色视频占位）
- 再不满足则生成占位文件，仅用于 `file_added` 流程验证

原生录屏单独运行：

```
./run_native_recorder.sh /tmp/aegis_screen.mp4 3
```
