use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::env;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

fn main() {
    if let Err(err) = run() {
        eprintln!("FAIL: {err}");
        std::process::exit(1);
    }
    println!("PASS");
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_default();
    if command != "verify" {
        return Err("usage: aegis-verifier verify <bundle_path>".to_string());
    }
    let bundle_path = args.next().ok_or("missing bundle path")?;
    let bundle_path = PathBuf::from(bundle_path);

    let session_path = bundle_path.join("session.json");
    let events_path = bundle_path.join("events.jsonl");
    let manifest_path = bundle_path.join("manifest.json");

    ensure_exists(&session_path)?;
    ensure_exists(&events_path)?;
    ensure_exists(&manifest_path)?;

    let (last_hash, events_count) = verify_event_sequence(&events_path)?;
    if events_count == 0 {
        return Err("events.jsonl is empty".to_string());
    }

    let manifest = read_manifest(&manifest_path)?;
    verify_manifest_files(&bundle_path, &manifest)?;

    let events_hash = sha256_hex(&fs::read(&events_path).map_err(|err| err.to_string())?);
    let manifest_events_hash = get_manifest_string(&manifest, "events_hash")?;
    if events_hash != manifest_events_hash {
        return Err("events_hash mismatch".to_string());
    }

    let manifest_final_hash = get_manifest_string(&manifest, "final_hash")?;
    if manifest_final_hash != last_hash {
        return Err("final_hash mismatch".to_string());
    }

    Ok(())
}

fn ensure_exists(path: &Path) -> Result<(), String> {
    if path.exists() {
        Ok(())
    } else {
        Err(format!("missing required file: {}", path.display()))
    }
}

fn read_manifest(path: &Path) -> Result<Value, String> {
    let content = fs::read_to_string(path).map_err(|err| err.to_string())?;
    serde_json::from_str(&content).map_err(|err| format!("parse manifest: {err}"))
}

fn get_manifest_string(manifest: &Value, key: &str) -> Result<String, String> {
    manifest
        .get(key)
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
        .ok_or_else(|| format!("manifest missing {key}"))
}

fn verify_manifest_files(bundle_path: &Path, manifest: &Value) -> Result<(), String> {
    let files = manifest
        .get("files")
        .and_then(|value| value.as_array())
        .ok_or("manifest missing files")?;

    for entry in files {
        let rel_path = entry
            .get("rel_path")
            .and_then(|value| value.as_str())
            .ok_or("manifest file missing rel_path")?;
        let full_path = bundle_path.join(rel_path);
        if !full_path.exists() {
            return Err(format!("missing file listed in manifest: {rel_path}"));
        }
    }

    Ok(())
}

fn verify_event_sequence(path: &Path) -> Result<(String, u64), String> {
    let file = File::open(path).map_err(|err| format!("open events: {err}"))?;
    let reader = BufReader::new(file);
    let mut expected_seq: u64 = 1;
    let mut last_hash = String::new();
    let mut count = 0;

    for (index, line) in reader.lines().enumerate() {
        let line = line.map_err(|err| format!("read events line: {err}"))?;
        if line.trim().is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(&line)
            .map_err(|err| format!("parse events line {}: {err}", index + 1))?;
        let seq = value
            .get("seq")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| format!("missing seq at line {}", index + 1))?;

        if seq != expected_seq {
            return Err(format!(
                "seq discontinuity at line {}: expected {}, got {}",
                index + 1,
                expected_seq,
                seq
            ));
        }

        let ts = value
            .get("ts")
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("missing ts at line {}", index + 1))?;
        let event_type = value
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("missing type at line {}", index + 1))?;
        let payload = value
            .get("payload")
            .ok_or_else(|| format!("missing payload at line {}", index + 1))?;
        let prev_hash = value
            .get("prev_hash")
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("missing prev_hash at line {}", index + 1))?;
        let hash = value
            .get("hash")
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("missing hash at line {}", index + 1))?;

        if expected_seq == 1 {
            if !prev_hash.is_empty() {
                return Err(format!("prev_hash should be empty at line {}", index + 1));
            }
        } else if prev_hash != last_hash {
            return Err(format!("prev_hash mismatch at line {}", index + 1));
        }

        let payload = canonicalize_value(payload);
        let hash_input = serde_json::json!({
            "seq": seq,
            "ts": ts,
            "type": event_type,
            "payload": payload,
            "prev_hash": prev_hash,
        });
        let expected_hash = sha256_hex(canonical_json_string(&hash_input).as_bytes());
        if expected_hash != hash {
            return Err(format!("hash mismatch at line {}", index + 1));
        }

        last_hash = hash.to_string();
        expected_seq += 1;
        count += 1;
    }

    Ok((last_hash, count))
}

fn canonicalize_value(value: &Value) -> Value {
    match value {
        Value::Array(items) => {
            let mapped: Vec<Value> = items.iter().map(canonicalize_value).collect();
            Value::Array(mapped)
        }
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let mut new_map = Map::new();
            for key in keys {
                if let Some(value) = map.get(key) {
                    new_map.insert(key.clone(), canonicalize_value(value));
                }
            }
            Value::Object(new_map)
        }
        other => other.clone(),
    }
}

fn canonical_json_string(value: &Value) -> String {
    let canonical = canonicalize_value(value);
    serde_json::to_string(&canonical).unwrap_or_default()
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    bytes_to_hex(&digest)
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{:02x}", byte));
    }
    out
}
