use aegis_core::SessionWriter;
use serde::Deserialize;
use serde_json::Value;
use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Component, Path, PathBuf};

#[derive(Deserialize)]
struct IncomingMessage {
    #[serde(rename = "type")]
    message_type: String,
    #[serde(default)]
    payload: Value,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("FAIL: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let platform =
        args.next()
            .ok_or("usage: aegis-core-server <platform> <app_version> [save_dir] [addr]")?;
    let app_version = args.next().ok_or("missing app_version")?;

    let mut save_dir: Option<PathBuf> = None;
    let mut addr: Option<String> = None;
    for arg in args {
        if save_dir.is_none() && addr.is_none() {
            if looks_like_addr(&arg) {
                addr = Some(arg);
            } else {
                save_dir = Some(PathBuf::from(arg));
            }
        } else if save_dir.is_none() {
            save_dir = Some(PathBuf::from(arg));
        } else if addr.is_none() {
            addr = Some(arg);
        } else {
            return Err("too many arguments".to_string());
        }
    }

    let save_dir = save_dir.unwrap_or_else(default_save_dir);
    let addr = addr
        .or_else(|| env::var("AEGIS_CORE_ADDR").ok())
        .unwrap_or_else(|| "127.0.0.1:7878".to_string());
    let mut writer = SessionWriter::start_session(&save_dir, &platform, &app_version)
        .map_err(|err| format!("start session: {err}"))?;

    eprintln!(
        "Session started at {} (listening on {})",
        writer.session_dir().display(),
        addr
    );

    let listener =
        TcpListener::bind(&addr).map_err(|err| format!("bind {}: {err}", addr))?;

    loop {
        let (stream, _) = listener
            .accept()
            .map_err(|err| format!("accept connection: {err}"))?;
        let should_stop = handle_connection(stream, &mut writer)?;
        if should_stop {
            break;
        }
    }
    Ok(())
}

fn looks_like_addr(arg: &str) -> bool {
    arg.contains(':') && !arg.contains('/') && !arg.contains('\\')
}

fn default_save_dir() -> PathBuf {
    let home = env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from);
    if let Some(home) = home {
        return home.join("Downloads");
    }
    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn parse_file_payload(payload: &Value) -> Result<(String, String, PathBuf), String> {
    let rel_path = payload
        .get("rel_path")
        .and_then(|value| value.as_str())
        .ok_or("file_added missing rel_path")?;
    let kind = payload
        .get("kind")
        .and_then(|value| value.as_str())
        .ok_or("file_added missing kind")?;
    let source_path = payload
        .get("source_path")
        .and_then(|value| value.as_str())
        .ok_or("file_added missing source_path")?;

    if !is_safe_rel_path(rel_path) {
        return Err("file_added rel_path must be relative and not contain '..'".to_string());
    }

    Ok((rel_path.to_string(), kind.to_string(), PathBuf::from(source_path)))
}

fn parse_shot_payload(payload: &Value) -> Result<(String, PathBuf), String> {
    let rel_path = payload
        .get("rel_path")
        .and_then(|value| value.as_str())
        .ok_or("shot_saved missing rel_path")?;
    let source_path = payload
        .get("source_path")
        .and_then(|value| value.as_str())
        .ok_or("shot_saved missing source_path")?;

    if !is_safe_rel_path(rel_path) {
        return Err("shot_saved rel_path must be relative and not contain '..'".to_string());
    }

    Ok((rel_path.to_string(), PathBuf::from(source_path)))
}

fn is_safe_rel_path(path: &str) -> bool {
    let candidate = Path::new(path);
    if candidate.is_absolute() {
        return false;
    }
    for component in candidate.components() {
        if let Component::ParentDir = component {
            return false;
        }
    }
    true
}

fn copy_into_bundle(session_dir: &Path, rel_path: &str, source_path: &Path) -> Result<(), String> {
    if !source_path.exists() {
        return Err(format!("source file missing: {}", source_path.display()));
    }
    let destination = session_dir.join(rel_path);
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("create dirs: {err}"))?;
    }
    fs::copy(source_path, destination).map_err(|err| format!("copy file: {err}"))?;
    Ok(())
}

fn handle_connection(mut stream: TcpStream, writer: &mut SessionWriter) -> Result<bool, String> {
    let reader = BufReader::new(stream.try_clone().map_err(|err| err.to_string())?);

    for (index, line) in reader.lines().enumerate() {
        let line = line.map_err(|err| format!("read line {}: {err}", index + 1))?;
        if line.trim().is_empty() {
            continue;
        }
        let msg: IncomingMessage = serde_json::from_str(&line)
            .map_err(|err| format!("parse message {}: {err}", index + 1))?;

        if msg.message_type == "stop" {
            let reason = msg
                .payload
                .get("reason")
                .and_then(|value| value.as_str())
                .unwrap_or("unknown");
            writer
                .stop_session(reason)
                .map_err(|err| format!("stop session: {err}"))?;
            stream
                .write_all(b"OK\n")
                .map_err(|err| format!("write response: {err}"))?;
            return Ok(true);
        }

        if msg.message_type == "file_added" {
            let (rel_path, kind, source_path) = parse_file_payload(&msg.payload)?;
            copy_into_bundle(writer.session_dir(), &rel_path, &source_path)?;
            writer
                .append_event(
                    "file_added",
                    serde_json::json!({ "rel_path": rel_path, "kind": kind }),
                )
                .map_err(|err| format!("append event: {err}"))?;
            stream
                .write_all(b"OK\n")
                .map_err(|err| format!("write response: {err}"))?;
            continue;
        }

        if msg.message_type == "shot_saved" {
            let (rel_path, source_path) = parse_shot_payload(&msg.payload)?;
            copy_into_bundle(writer.session_dir(), &rel_path, &source_path)?;
            writer
                .append_event("shot_saved", serde_json::json!({ "rel_path": rel_path }))
                .map_err(|err| format!("append event: {err}"))?;
            stream
                .write_all(b"OK\n")
                .map_err(|err| format!("write response: {err}"))?;
            continue;
        }

        writer
            .append_event(&msg.message_type, msg.payload)
            .map_err(|err| format!("append event: {err}"))?;
        stream
            .write_all(b"OK\n")
            .map_err(|err| format!("write response: {err}"))?;
    }

    Ok(false)
}
