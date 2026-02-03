use serde::Serialize;
use serde_json::json;
use std::env;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

#[derive(Serialize)]
struct Message {
    #[serde(rename = "type")]
    message_type: String,
    payload: serde_json::Value,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("FAIL: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let command =
        args.next()
            .ok_or("usage: aegis-collector-cli <focus|file|shot|input|stop> [args]")?;
    let addr = env::var("AEGIS_CORE_ADDR").unwrap_or_else(|_| "127.0.0.1:7878".to_string());

    let message = match command.as_str() {
        "focus" => {
            let app_id = args.next().ok_or("missing app_id")?;
            let app_name = args.next().ok_or("missing app_name")?;
            let window_title = args.next();
            let mut payload = json!({
                "app_id": app_id,
                "app_name": app_name,
            });
            if let Some(title) = window_title {
                payload["window_title"] = json!(title);
            }
            Message {
                message_type: "app_focus_changed".to_string(),
                payload,
            }
        }
        "file" => {
            let source_path = args.next().ok_or("missing source_path")?;
            let rel_path = args.next().ok_or("missing rel_path")?;
            let kind = args.next().ok_or("missing kind")?;
            Message {
                message_type: "file_added".to_string(),
                payload: json!({
                    "source_path": source_path,
                    "rel_path": rel_path,
                    "kind": kind,
                }),
            }
        }
        "shot" => {
            let source_path = args.next().ok_or("missing source_path")?;
            let rel_path = args.next().ok_or("missing rel_path")?;
            Message {
                message_type: "shot_saved".to_string(),
                payload: json!({
                    "source_path": source_path,
                    "rel_path": rel_path,
                }),
            }
        }
        "input" => {
            let interval_ms = args.next().ok_or("missing interval_ms")?;
            let key_count = args.next().ok_or("missing key_count")?;
            let backspace_count = args.next().ok_or("missing backspace_count")?;
            let paste_count = args.next().ok_or("missing paste_count")?;
            Message {
                message_type: "input_stats".to_string(),
                payload: json!({
                    "interval_ms": interval_ms.parse::<u64>().map_err(|_| "invalid interval_ms")?,
                    "key_count": key_count.parse::<u64>().map_err(|_| "invalid key_count")?,
                    "backspace_count": backspace_count.parse::<u64>().map_err(|_| "invalid backspace_count")?,
                    "paste_count": paste_count.parse::<u64>().map_err(|_| "invalid paste_count")?,
                }),
            }
        }
        "stop" => {
            let reason = args.next().unwrap_or_else(|| "user".to_string());
            Message {
                message_type: "stop".to_string(),
                payload: json!({ "reason": reason }),
            }
        }
        _ => return Err("usage: aegis-collector-cli <focus|file|shot|input|stop> [args]".to_string()),
    };

    let mut stream =
        TcpStream::connect(&addr).map_err(|err| format!("connect {addr}: {err}"))?;
    serde_json::to_writer(&mut stream, &message)
        .map_err(|err| format!("write message: {err}"))?;
    stream
        .write_all(b"\n")
        .map_err(|err| format!("write newline: {err}"))?;

    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader
        .read_line(&mut response)
        .map_err(|err| format!("read response: {err}"))?;
    if !response.starts_with("OK") {
        return Err(format!("unexpected response: {response}"));
    }

    Ok(())
}
