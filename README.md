# AEGISTRACE

AEGISTRACE is a cross-platform evidence capture system focused on a unified
bundle format, tamper detection, and independent verification.

## Goals

- Keep the evidence format, hash chain, and verifier consistent across OSes
- Allow native collectors per platform while sharing Rust core/verifier
- Provide a minimal GUI shell (optional) for start/stop workflows

## Repository Layout

```
AEGISTRACE/
  crates/
    aegis-core/            # Rust: schema, events, hash chain, bundle writer
    aegis-core-server/     # Rust: TCP core server (collector IPC)
    aegis-collector-cli/   # Rust: minimal collector CLI
    aegis-verifier/        # Rust: CLI verifier
  apps/
    aegis-tauri/           # Tauri GUI shell
  collectors/
    macos/                 # Swift: screen/app/input/network (later)
    windows/               # C# or C++ collectors
    linux/                 # Rust/C collectors
  spec/
    evidence_bundle.md     # Evidence bundle spec (public)
  scripts/
    build_macos.sh
    build_windows.ps1
    build_linux.sh
```

## Evidence Bundle (High Level)

```
Evidence_YYYYMMDD_HHMMSS/
  session.json
  events.jsonl
  manifest.json
  files/
    screen.mp4             # optional
    shots/                 # optional
```

Minimal `events.jsonl` fields:

- `seq`: increasing sequence number
- `ts`: UTC timestamp (ISO8601)
- `type`: event type (e.g. `session_started`, `app_focus_changed`)
- `payload`: event data (JSON object)
- `prev_hash` / `hash`: hash chain fields

## Development Plan

See `执行计划` for the step-by-step roadmap and validation checks.

## Technical Guide

See `AEGISTRACE 全栈技术指导` for architecture, IPC strategy, and rollout flow.

## Releases (Phase 5)

GitHub Actions builds artifacts on tag pushes (`v*`) for macOS/Linux/Windows.

## macOS App Bundle

To create a single `.app` bundle locally:

```
./scripts/macos_app_bundle.sh
```

The bundle will be at `dist/macos/AEGISTRACE.app`.

## GUI Shell (Tauri)

The Tauri UI lives in `apps/aegis-tauri`.
Run it from that directory with `cargo tauri dev`.
