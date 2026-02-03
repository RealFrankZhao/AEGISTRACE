use chrono::{DateTime, SecondsFormat, Utc};
use serde::Serialize;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};

#[derive(Serialize)]
struct EventRecord {
    seq: u64,
    ts: String,
    #[serde(rename = "type")]
    event_type: String,
    payload: Value,
    prev_hash: String,
    hash: String,
}

#[derive(Serialize)]
struct SessionRecord {
    started_at: DateTime<Utc>,
    ended_at: Option<DateTime<Utc>>,
    platform: String,
    app_version: String,
    bundle_dir: String,
}

#[derive(Serialize)]
struct ManifestRecord {
    schema_version: u32,
    events_hash: String,
    final_hash: String,
    files: Vec<ManifestFile>,
}

#[derive(Serialize)]
struct ManifestFile {
    rel_path: String,
    hash: String,
}

pub struct SessionWriter {
    session_dir: PathBuf,
    events_writer: BufWriter<File>,
    events_hasher: Sha256,
    last_hash: Option<String>,
    seq: u64,
    started_at: DateTime<Utc>,
    save_dir: PathBuf,
    platform: String,
    app_version: String,
}

impl SessionWriter {
    pub fn start_session(
        save_dir: impl AsRef<Path>,
        platform: &str,
        app_version: &str,
    ) -> io::Result<Self> {
        let started_at = Utc::now();
        let save_dir = save_dir.as_ref().to_path_buf();
        let session_dir = save_dir
            .join(format!("Evidence_{}", started_at.format("%Y%m%d_%H%M%S")));
        fs::create_dir_all(session_dir.join("files"))?;

        let events_path = session_dir.join("events.jsonl");
        let events_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(events_path)?;

        let mut writer = Self {
            session_dir,
            events_writer: BufWriter::new(events_file),
            events_hasher: Sha256::new(),
            last_hash: None,
            seq: 1,
            started_at,
            save_dir,
            platform: platform.to_string(),
            app_version: app_version.to_string(),
        };

        writer.append_event(
            "session_started",
            serde_json::json!({
                "save_dir": writer.save_dir.to_string_lossy(),
                "platform": writer.platform.clone(),
                "app_version": writer.app_version.clone(),
            }),
        )?;

        Ok(writer)
    }

    pub fn append_event(&mut self, event_type: &str, payload: Value) -> io::Result<()> {
        let ts = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
        let payload = canonicalize_value(&payload);
        let prev_hash = self.last_hash.clone().unwrap_or_default();

        let hash_input = serde_json::json!({
            "seq": self.seq,
            "ts": ts,
            "type": event_type,
            "payload": payload,
            "prev_hash": prev_hash,
        });
        let hash = sha256_hex(canonical_json_string(&hash_input).as_bytes());

        let record = EventRecord {
            seq: self.seq,
            ts,
            event_type: event_type.to_string(),
            payload,
            prev_hash,
            hash: hash.clone(),
        };
        self.seq += 1;

        let line = serde_json::to_string(&record)?;
        self.events_writer.write_all(line.as_bytes())?;
        self.events_writer.write_all(b"\n")?;
        self.events_writer.flush()?;

        self.events_hasher.update(line.as_bytes());
        self.events_hasher.update(b"\n");
        self.last_hash = Some(hash);
        Ok(())
    }

    pub fn stop_session(&mut self, reason: &str) -> io::Result<()> {
        self.append_event(
            "session_stopped",
            serde_json::json!({ "reason": reason }),
        )?;

        let session_record = SessionRecord {
            started_at: self.started_at,
            ended_at: Some(Utc::now()),
            platform: self.platform.clone(),
            app_version: self.app_version.clone(),
            bundle_dir: self
                .session_dir
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        };
        let session_path = self.session_dir.join("session.json");
        let mut session_file = File::create(&session_path)?;
        serde_json::to_writer_pretty(&mut session_file, &session_record)?;
        session_file.write_all(b"\n")?;

        let events_hash = finalize_hasher(&self.events_hasher);
        let final_hash = self
            .last_hash
            .clone()
            .unwrap_or_else(|| "".to_string());

        let mut files = Vec::new();
        files.push(ManifestFile {
            rel_path: "session.json".to_string(),
            hash: sha256_hex(&fs::read(&session_path)?),
        });
        let events_path = self.session_dir.join("events.jsonl");
        files.push(ManifestFile {
            rel_path: "events.jsonl".to_string(),
            hash: sha256_hex(&fs::read(&events_path)?),
        });

        let files_root = self.session_dir.join("files");
        let mut extra_files = Vec::new();
        collect_files(&files_root, &self.session_dir, &mut extra_files)?;
        for rel_path in extra_files {
            let full_path = self.session_dir.join(&rel_path);
            files.push(ManifestFile {
                rel_path: rel_path.to_string_lossy().to_string(),
                hash: sha256_hex(&fs::read(full_path)?),
            });
        }
        files.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));

        let manifest = ManifestRecord {
            schema_version: 1,
            events_hash,
            final_hash,
            files,
        };
        let manifest_path = self.session_dir.join("manifest.json");
        let mut manifest_file = File::create(manifest_path)?;
        serde_json::to_writer_pretty(&mut manifest_file, &manifest)?;
        manifest_file.write_all(b"\n")?;
        Ok(())
    }

    pub fn session_dir(&self) -> &Path {
        &self.session_dir
    }
}

fn collect_files(dir: &Path, base: &Path, out: &mut Vec<PathBuf>) -> io::Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, base, out)?;
        } else if let Ok(rel) = path.strip_prefix(base) {
            out.push(rel.to_path_buf());
        }
    }
    Ok(())
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

fn finalize_hasher(hasher: &Sha256) -> String {
    let clone = hasher.clone();
    let digest = clone.finalize();
    bytes_to_hex(&digest)
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
