//! PreToolUse hook: stdin JSON -> decision.
//!
//! Allow = exit 0, no stdout (passthrough).
//! Deny  = exit 0, stdout JSON with hookSpecificOutput.
//! Never exit non-zero for policy decisions; on ANY internal error: allow
//! (fail-open — the hook must never block a read it cannot analyze).

use crate::config::Config;
use crate::outline;
use crate::store::{self, SessionRecord, StatRecord, Store};
use serde::Deserialize;
use serde_json::json;
use std::fs;
use std::io::Read as _;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct HookInput {
    #[serde(default)]
    session_id: String,
    #[serde(default)]
    tool_name: String,
    #[serde(default)]
    tool_input: ToolInput,
}

#[derive(Debug, Default, Deserialize)]
struct ToolInput {
    #[serde(default)]
    file_path: Option<String>,
    #[serde(default)]
    offset: Option<serde_json::Value>,
    #[serde(default)]
    limit: Option<serde_json::Value>,
}

/// Hard cap for the rule-6 outline read. Files larger than this are allowed
/// through untouched (fail-open) so the hook never allocates unboundedly or
/// stalls on a multi-hundred-MB text file.
pub const OUTLINE_MAX_BYTES: u64 = 8 * 1024 * 1024;

pub const DUP_REASON: &str = "stk: file unchanged since stk last saw it this session (hash match). Re-read with offset/limit if you need to re-check a specific range.";

/// Build the deny JSON payload (the golden output contract).
pub fn deny_json(reason: &str) -> String {
    json!({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "deny",
            "permissionDecisionReason": reason
        }
    })
    .to_string()
}

/// Decision result: `None` = allow (no stdout), `Some(json)` = deny payload.
pub fn decide(raw_input: &str, config: &Config, store_root: PathBuf) -> Option<String> {
    // Fail-open: malformed stdin -> allow.
    let input: HookInput = serde_json::from_str(raw_input).ok()?;
    if input.tool_name != "Read" {
        return None;
    }
    let file_path = input.tool_input.file_path.clone()?;

    // Rule 1: model already scoping with offset/limit — never fight it.
    if input.tool_input.offset.is_some() || input.tool_input.limit.is_some() {
        return None;
    }

    // Rule 2: missing / unreadable / not a regular file -> allow.
    let meta = fs::metadata(&file_path).ok()?;
    if !meta.is_file() {
        return None;
    }
    let size = meta.len();

    // Exclude globs: always allowed through.
    if config.is_excluded(&file_path) {
        return None;
    }

    // Rule 3: binary or image/PDF/notebook extension -> allow.
    if outline::is_image_pdf_or_notebook(&file_path) {
        return None;
    }
    let head = read_head(&file_path, 8192)?;
    if head.contains(&0u8) {
        return None;
    }

    let store = Store::new(store_root);
    store.prune_old_sessions();

    // Hash computed only for files <= 4 MiB (else skip dup layer).
    let content: Option<String> = if size <= store::HASH_MAX_BYTES {
        fs::read_to_string(&file_path).ok()
    } else {
        None
    };
    let hash = content
        .as_deref()
        .map(|c| store::sha1_hex(c.as_bytes()))
        .unwrap_or_default();

    // Rule 4: small file -> allow + record hash.
    if size <= config.clamp_threshold {
        let _ = store.record_session(
            &input.session_id,
            &SessionRecord {
                ts: store::now_ts(),
                file: file_path.clone(),
                size,
                hash,
                action: "allow".into(),
            },
        );
        return None;
    }

    // Rule 5: same path + same content hash already recorded this session -> deny (dup).
    if config.dedup && !hash.is_empty() {
        if store.latest_hash(&input.session_id, &file_path).as_deref() == Some(hash.as_str()) {
            let _ = store.record_session(
                &input.session_id,
                &SessionRecord {
                    ts: store::now_ts(),
                    file: file_path.clone(),
                    size,
                    hash: hash.clone(),
                    action: "dup".into(),
                },
            );
            let _ = store.record_stat(&StatRecord {
                ts: store::now_ts(),
                file: file_path.clone(),
                file_bytes: size,
                sent_bytes: DUP_REASON.len() as u64,
                kind: "dup".into(),
            });
            return Some(deny_json(DUP_REASON));
        }
    }

    // Rule 6: big file, first sight -> deny with outline.
    // Never read unboundedly: above OUTLINE_MAX_BYTES, allow (fail-open).
    let text = match content {
        Some(c) => c,
        None if size <= OUTLINE_MAX_BYTES => fs::read_to_string(&file_path).ok()?,
        None => return None,
    };
    let reason = outline::generate(
        &file_path,
        &text,
        size,
        config.clamp_threshold,
        config.outline_max_lines,
    );
    let _ = store.record_session(
        &input.session_id,
        &SessionRecord {
            ts: store::now_ts(),
            file: file_path.clone(),
            size,
            hash,
            action: "clamp".into(),
        },
    );
    let _ = store.record_stat(&StatRecord {
        ts: store::now_ts(),
        file: file_path,
        file_bytes: size,
        sent_bytes: reason.len() as u64,
        kind: "clamp".into(),
    });
    Some(deny_json(&reason))
}

fn read_head(path: &str, n: usize) -> Option<Vec<u8>> {
    let mut f = fs::File::open(path).ok()?;
    let mut buf = vec![0u8; n];
    let mut read = 0usize;
    loop {
        match std::io::Read::read(&mut f, &mut buf[read..]) {
            Ok(0) => break,
            Ok(k) => read += k,
            Err(_) => return None,
        }
        if read == n {
            break;
        }
    }
    buf.truncate(read);
    Some(buf)
}

/// Entry point for `stk hook claude`: read all of stdin, print deny JSON if any.
/// Always returns exit code 0.
pub fn run() -> i32 {
    let mut raw = String::new();
    if std::io::stdin().read_to_string(&mut raw).is_err() {
        return 0; // fail-open
    }
    // Fail-open is structural: any panic inside config/store/decide must not
    // escape as a nonzero exit or stderr noise.
    std::panic::set_hook(Box::new(|_| {}));
    let result = std::panic::catch_unwind(|| {
        let config = Config::load();
        decide(&raw, &config, crate::config::store_root())
    });
    if let Ok(Some(payload)) = result {
        println!("{payload}");
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use crate::testutil::tempdir;

    fn write_file(dir: &std::path::Path, name: &str, bytes: &[u8]) -> String {
        let p = dir.join(name);
        let mut f = fs::File::create(&p).unwrap();
        f.write_all(bytes).unwrap();
        p.to_string_lossy().to_string()
    }

    fn input_json(session: &str, path: &str) -> String {
        json!({"session_id": session, "tool_name": "Read", "tool_input": {"file_path": path}})
            .to_string()
    }

    fn cfg() -> Config {
        Config::default()
    }

    #[test]
    fn rule1_offset_or_limit_allows() {
        let dir = tempdir().unwrap();
        let big = "x\n".repeat(20000);
        let path = write_file(dir.path(), "big.txt", big.as_bytes());
        let raw = json!({"session_id":"s","tool_name":"Read",
            "tool_input":{"file_path": path, "offset": 100, "limit": 50}})
        .to_string();
        assert_eq!(decide(&raw, &cfg(), dir.path().join("store")), None);
    }

    #[test]
    fn rule2_missing_file_allows() {
        let dir = tempdir().unwrap();
        let raw = input_json("s", "C:\\definitely\\not\\a\\real\\file.xyz");
        assert_eq!(decide(&raw, &cfg(), dir.path().join("store")), None);
    }

    #[test]
    fn rule3_binary_and_image_ext_allow() {
        let dir = tempdir().unwrap();
        let mut bin = vec![b'a'; 30000];
        bin[5] = 0; // NUL in first 8KB
        let bin_path = write_file(dir.path(), "blob.dat", &bin);
        assert_eq!(decide(&input_json("s", &bin_path), &cfg(), dir.path().join("store")), None);

        let big_text = "y\n".repeat(20000);
        let img_path = write_file(dir.path(), "pic.png", big_text.as_bytes());
        assert_eq!(decide(&input_json("s", &img_path), &cfg(), dir.path().join("store")), None);
    }

    #[test]
    fn rule4_small_file_allows_and_records() {
        let dir = tempdir().unwrap();
        let path = write_file(dir.path(), "small.rs", b"fn main() {}\n");
        let root = dir.path().join("store");
        assert_eq!(decide(&input_json("s4", &path), &cfg(), root.clone()), None);
        let store = Store::new(root);
        assert!(store.latest_hash("s4", &path).is_some());
    }

    #[test]
    fn rule5_dup_denies_with_dup_reason() {
        let dir = tempdir().unwrap();
        let big = "fn main() {}\n".repeat(3000); // > 16 KiB
        let path = write_file(dir.path(), "big.rs", big.as_bytes());
        let root = dir.path().join("store");

        // First sight: clamp (outline deny).
        let first = decide(&input_json("s5", &path), &cfg(), root.clone()).unwrap();
        assert!(first.contains("stk clamp:"));
        // Second sight, unchanged: dup deny.
        let second = decide(&input_json("s5", &path), &cfg(), root.clone()).unwrap();
        assert!(second.contains("file unchanged since stk last saw it"), "{second}");
        // Different session: back to outline clamp.
        let other = decide(&input_json("other", &path), &cfg(), root.clone()).unwrap();
        assert!(other.contains("stk clamp:"));

        // Changed content: hash differs -> outline again, not dup.
        let big2 = format!("{big}\n// changed\n");
        fs::write(&path, &big2).unwrap();
        let third = decide(&input_json("s5", &path), &cfg(), root).unwrap();
        assert!(third.contains("stk clamp:"), "{third}");
    }

    #[test]
    fn rule6_big_file_denies_with_outline() {
        let dir = tempdir().unwrap();
        let big = "fn main() {}\n".repeat(3000);
        let path = write_file(dir.path(), "huge.rs", big.as_bytes());
        let root = dir.path().join("store");
        let out = decide(&input_json("s6", &path), &cfg(), root.clone()).unwrap();

        // Golden JSON contract.
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        let hso = &v["hookSpecificOutput"];
        assert_eq!(hso["hookEventName"], "PreToolUse");
        assert_eq!(hso["permissionDecision"], "deny");
        let reason = hso["permissionDecisionReason"].as_str().unwrap();
        assert!(reason.starts_with("stk clamp:"));
        assert!(reason.contains("offset=1, limit=3000"));
        assert_eq!(v.as_object().unwrap().len(), 1);
        assert_eq!(hso.as_object().unwrap().len(), 3);

        // Stat recorded.
        let stats = Store::new(root).read_stats();
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].kind, "clamp");
        assert!(stats[0].file_bytes > stats[0].sent_bytes);
    }

    #[test]
    fn rule6_oversized_text_file_allows() {
        let dir = tempdir().unwrap();
        let big = "x".repeat(OUTLINE_MAX_BYTES as usize + 1024);
        let path = write_file(dir.path(), "huge.log", big.as_bytes());
        assert_eq!(decide(&input_json("s", &path), &cfg(), dir.path().join("store")), None);
    }

    #[test]
    fn golden_deny_json_exact() {
        let got = deny_json("why");
        let want = r#"{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"why"}}"#;
        assert_eq!(got, want);
    }

    #[test]
    fn fail_open_on_malformed_stdin() {
        let dir = tempdir().unwrap();
        for raw in ["", "not json", "{\"tool_name\":", "42", "{\"tool_name\":\"Read\"}"] {
            assert_eq!(decide(raw, &cfg(), dir.path().join("store")), None, "input: {raw:?}");
        }
    }

    #[test]
    fn non_read_tool_allows() {
        let dir = tempdir().unwrap();
        let raw = json!({"session_id":"s","tool_name":"Bash","tool_input":{"command":"ls"}}).to_string();
        assert_eq!(decide(&raw, &cfg(), dir.path().join("store")), None);
    }

    #[test]
    fn exclude_glob_allows_big_file() {
        let dir = tempdir().unwrap();
        let big = "l\n".repeat(20000);
        let path = write_file(dir.path(), "Cargo.lock", big.as_bytes());
        let mut config = Config::default();
        config.exclude = vec!["*.lock".into()];
        assert_eq!(decide(&input_json("s", &path), &config, dir.path().join("store")), None);
    }

    #[test]
    fn dedup_disabled_skips_dup_layer() {
        let dir = tempdir().unwrap();
        let big = "fn main() {}\n".repeat(3000);
        let path = write_file(dir.path(), "big.rs", big.as_bytes());
        let root = dir.path().join("store");
        let mut config = Config::default();
        config.dedup = false;
        let first = decide(&input_json("s", &path), &config, root.clone()).unwrap();
        let second = decide(&input_json("s", &path), &config, root).unwrap();
        assert!(first.contains("stk clamp:"));
        assert!(second.contains("stk clamp:"), "dup layer should be off: {second}");
    }
}
