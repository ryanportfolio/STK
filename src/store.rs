//! Session store + stats, append-only JSONL under the store root
//! (see `config::store_root()`; overridable via `STK_DATA_DIR`).
//!
//! Layout:
//!   <root>/sessions/<session_id>.jsonl   one record per decision
//!   <root>/stats.jsonl                   one record per clamp/dup

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Hash only files at or under this size; above it the dup layer is skipped.
pub const HASH_MAX_BYTES: u64 = 4 * 1024 * 1024;
/// Session files older than this many days are pruned on startup.
pub const PRUNE_DAYS: u64 = 14;

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionRecord {
    pub ts: u64,
    pub file: String,
    pub size: u64,
    pub hash: String,
    pub action: String, // "allow" | "clamp" | "dup"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatRecord {
    pub ts: u64,
    pub file: String,
    pub file_bytes: u64,
    pub sent_bytes: u64,
    pub kind: String, // "clamp" | "dup"
}

pub fn now_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub struct Store {
    root: PathBuf,
}

impl Store {
    pub fn new(root: PathBuf) -> Store {
        Store { root }
    }

    pub fn sessions_dir(&self) -> PathBuf {
        self.root.join("sessions")
    }

    pub fn stats_path(&self) -> PathBuf {
        self.root.join("stats.jsonl")
    }

    fn session_path(&self, session_id: &str) -> PathBuf {
        // Sanitize: session ids should be simple tokens; strip path separators.
        let safe: String = session_id
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
            .collect();
        self.sessions_dir().join(format!("{safe}.jsonl"))
    }

    fn append_line(path: &Path, line: &str) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut f = fs::OpenOptions::new().create(true).append(true).open(path)?;
        // Single write_all so concurrent hook processes cannot interleave the
        // record body and its trailing newline (writeln! issues two writes).
        f.write_all(format!("{line}\n").as_bytes())
    }

    pub fn record_session(&self, session_id: &str, rec: &SessionRecord) -> std::io::Result<()> {
        let line = serde_json::to_string(rec).map_err(std::io::Error::other)?;
        Self::append_line(&self.session_path(session_id), &line)
    }

    pub fn record_stat(&self, rec: &StatRecord) -> std::io::Result<()> {
        let line = serde_json::to_string(rec).map_err(std::io::Error::other)?;
        Self::append_line(&self.stats_path(), &line)
    }

    /// Latest recorded hash for `file` in this session (scan lines, last wins).
    pub fn latest_hash(&self, session_id: &str, file: &str) -> Option<String> {
        let text = fs::read_to_string(self.session_path(session_id)).ok()?;
        let mut latest: Option<String> = None;
        for line in text.lines() {
            if let Ok(rec) = serde_json::from_str::<SessionRecord>(line) {
                if rec.file == file && !rec.hash.is_empty() {
                    latest = Some(rec.hash);
                }
            }
        }
        latest
    }

    /// Delete session files older than `PRUNE_DAYS` (by modified time).
    pub fn prune_old_sessions(&self) {
        let Ok(entries) = fs::read_dir(self.sessions_dir()) else { return };
        let cutoff = std::time::Duration::from_secs(PRUNE_DAYS * 24 * 3600);
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(meta) = entry.metadata() else { continue };
            if !meta.is_file() {
                continue;
            }
            if let Ok(modified) = meta.modified() {
                if let Ok(age) = SystemTime::now().duration_since(modified) {
                    if age > cutoff {
                        let _ = fs::remove_file(&path);
                    }
                }
            }
        }
    }

    /// All stat records (unparseable lines skipped).
    pub fn read_stats(&self) -> Vec<StatRecord> {
        let Ok(text) = fs::read_to_string(self.stats_path()) else {
            return Vec::new();
        };
        text.lines()
            .filter_map(|l| serde_json::from_str::<StatRecord>(l).ok())
            .collect()
    }
}

pub fn sha1_hex(bytes: &[u8]) -> String {
    use sha1::{Digest, Sha1};
    let mut h = Sha1::new();
    h.update(bytes);
    let out = h.finalize();
    out.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::tempdir;

    fn rec(file: &str, hash: &str, action: &str) -> SessionRecord {
        SessionRecord {
            ts: now_ts(),
            file: file.into(),
            size: 10,
            hash: hash.into(),
            action: action.into(),
        }
    }

    #[test]
    fn dup_detection_latest_hash() {
        let dir = tempdir().unwrap();
        let store = Store::new(dir.path().to_path_buf());
        assert_eq!(store.latest_hash("s1", "a.txt"), None);

        store.record_session("s1", &rec("a.txt", "h1", "allow")).unwrap();
        store.record_session("s1", &rec("b.txt", "h2", "allow")).unwrap();
        assert_eq!(store.latest_hash("s1", "a.txt").as_deref(), Some("h1"));
        // Different session: isolated.
        assert_eq!(store.latest_hash("s2", "a.txt"), None);

        // Newer record for same file wins.
        store.record_session("s1", &rec("a.txt", "h3", "clamp")).unwrap();
        assert_eq!(store.latest_hash("s1", "a.txt").as_deref(), Some("h3"));
    }

    #[test]
    fn stats_roundtrip() {
        let dir = tempdir().unwrap();
        let store = Store::new(dir.path().to_path_buf());
        store
            .record_stat(&StatRecord {
                ts: 1,
                file: "x".into(),
                file_bytes: 1000,
                sent_bytes: 100,
                kind: "clamp".into(),
            })
            .unwrap();
        let stats = store.read_stats();
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].file_bytes, 1000);
        assert_eq!(stats[0].kind, "clamp");
    }

    #[test]
    fn prune_removes_old_files() {
        let dir = tempdir().unwrap();
        let store = Store::new(dir.path().to_path_buf());
        store.record_session("old", &rec("a", "h", "allow")).unwrap();
        store.record_session("new", &rec("a", "h", "allow")).unwrap();

        // Backdate the "old" session file 15 days.
        let old_path = store.sessions_dir().join("old.jsonl");
        let old_time = SystemTime::now() - std::time::Duration::from_secs(15 * 24 * 3600);
        let f = fs::File::options().append(true).open(&old_path).unwrap();
        f.set_modified(old_time).unwrap();
        drop(f);

        store.prune_old_sessions();
        assert!(!old_path.exists());
        assert!(store.sessions_dir().join("new.jsonl").exists());
    }

    #[test]
    fn sha1_known_vector() {
        assert_eq!(sha1_hex(b"abc"), "a9993e364706816aba3e25717850c26c9cd0d89d");
    }
}
