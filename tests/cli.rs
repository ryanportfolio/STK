//! Exercise the shipped executable and on-disk configuration, not just helpers.
#[path = "../src/testutil.rs"]
mod testutil;
use serde_json::{json, Value};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};

fn run(root: &Path, args: &[&str], input: Option<&Value>) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_stk"))
        .args(args)
        .env("STK_DATA_DIR", root.join("data"))
        .env("STK_CONFIG_FILE", root.join("config.toml"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    if let Some(input) = input {
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.to_string().as_bytes())
            .unwrap();
    }
    child.wait_with_output().unwrap()
}
fn success(output: &Output) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn client_totals_include_both_hooks_and_preserve_legacy_records() {
    let dir = testutil::tempdir().unwrap();
    let root = dir.path();
    fs::create_dir(root.join("data")).unwrap();
    fs::write(root.join("data/stats.jsonl"), concat!(
        "{\"ts\":1784592000,\"file\":\"old\",\"file_bytes\":1000,\"sent_bytes\":100,\"kind\":\"clamp\"}\n",
        "{\"ts\":1784592000,\"client\":\"future\",\"file\":\"other\",\"file_bytes\":100,\"sent_bytes\":150,\"kind\":\"dup\"}\n"
    )).unwrap();
    let path = root.join("large.txt");
    fs::write(&path, "some content for a large file\n".repeat(1500)).unwrap();
    for client in ["claude", "codex"] {
        let event = if client == "claude" {
            json!({"tool_name":"Read", "session_id":"stats", "tool_input":{"file_path":path}})
        } else {
            json!({"tool_name":"Bash", "session_id":"stats", "hook_event_name":"PreToolUse", "cwd":root,
                "tool_input":{"command":"Get-Content large.txt"}})
        };
        success(&run(root, &["hook", client], Some(&event)));
    }
    let output = run(root, &["gain", "--json"], None);
    success(&output);
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["clamps"], 3);
    assert_eq!(report["dup_hits"], 1);
    for client in ["claude", "codex", "legacy"] {
        assert_eq!(report["clients"][client]["clamps"], 1);
    }
    assert_eq!(report["clients"]["legacy"]["bytes_avoided"], 900);
    assert_eq!(report["clients"]["legacy"]["dup_hits"], 1);
    let total: u64 = report["clients"].as_object().unwrap().values()
        .map(|c| c["bytes_avoided"].as_u64().unwrap()).sum();
    assert_eq!(report["bytes_avoided"], total);
    assert_eq!(report["est_tokens"], total / 4);
    let daily: u64 = report["days"].as_array().unwrap().iter()
        .map(|d| d["bytes_avoided"].as_u64().unwrap()).sum();
    assert_eq!(daily, total);
    assert!(!String::from_utf8_lossy(&output.stdout).contains("large.txt"));
}

#[test]
fn both_hooks_and_range_recovery_in_real_processes() {
    let dir = testutil::tempdir().unwrap();
    let root = dir.path();
    let path = root.join("large file.rs");
    let content = (1..=3000)
        .map(|i| format!("fn item_{i}() {{}}\n"))
        .collect::<String>();
    fs::write(&path, &content).unwrap();
    let claude = json!({"session_id":"same", "tool_name":"Read", "tool_input":{"file_path":path}});
    let codex = json!({"session_id":"same", "hook_event_name":"PreToolUse", "cwd":root,
        "tool_name":"Bash", "tool_input":{"command":"Get-Content -LiteralPath 'large file.rs'"}});
    for (client, event) in [("claude", &claude), ("codex", &codex)] {
        let output = run(root, &["hook", client], Some(event));
        success(&output);
        let payload: Value = serde_json::from_slice(&output.stdout).unwrap();
        let reason = payload["hookSpecificOutput"]["permissionDecisionReason"]
            .as_str()
            .unwrap();
        assert_eq!(payload["hookSpecificOutput"]["permissionDecision"], "deny");
        assert!(reason.contains("stk clamp:"));
        assert!(reason.len() < content.len() / 4);
        assert_eq!(reason.contains("stk read"), client == "codex");
    }
    let output = run(
        root,
        &[
            "read",
            path.to_str().unwrap(),
            "--offset",
            "2100",
            "--limit",
            "2",
        ],
        None,
    );
    success(&output);
    assert_eq!(output.stdout, b"fn item_2100() {}\nfn item_2101() {}\n");
    let output = run(
        root,
        &["read", path.to_str().unwrap(), "--offset", "1"],
        None,
    );
    success(&output);
    assert_eq!(output.stdout, content.as_bytes());
    let repeat = run(root, &["hook", "claude"], Some(&claude));
    assert!(String::from_utf8_lossy(&repeat.stdout).contains("file unchanged"));
    let repeat = run(root, &["hook", "codex"], Some(&codex));
    assert!(String::from_utf8_lossy(&repeat.stdout).contains("stk clamp:"));
}

#[test]
fn hooks_pass_through_scoped_small_excluded_and_binary_reads() {
    let dir = testutil::tempdir().unwrap();
    let root = dir.path();
    fs::write(root.join("small.rs"), "fn tiny() {}\n").unwrap();
    fs::write(root.join("big.rs"), "fn item() {}\n".repeat(3000)).unwrap();
    fs::write(root.join("binary.dat"), vec![0u8; 30000]).unwrap();
    for command in [
        "Get-Content small.rs",
        "Get-Content binary.dat",
        "Get-Content big.rs -TotalCount 3",
        "cat big.rs | head",
        "cat missing.rs",
    ] {
        let event = json!({"hook_event_name":"PreToolUse","tool_name":"Bash","cwd":root,"tool_input":{"command":command}});
        let output = run(root, &["hook", "codex"], Some(&event));
        success(&output);
        assert!(output.stdout.is_empty(), "{command}");
    }
    fs::write(root.join("config.toml"), "exclude = ['*.rs']\n").unwrap();
    for client in ["claude", "codex"] {
        let event = if client == "claude" {
            json!({"tool_name":"Read","tool_input":{"file_path":root.join("big.rs")}})
        } else {
            json!({"hook_event_name":"PreToolUse","tool_name":"Bash","cwd":root,"tool_input":{"command":"Get-Content big.rs"}})
        };
        let output = run(root, &["hook", client], Some(&event));
        success(&output);
        assert!(output.stdout.is_empty());
    }
}

#[test]
fn installer_detects_both_backs_up_preserves_and_uninstalls() {
    let dir = testutil::tempdir().unwrap();
    let root = dir.path();
    fs::create_dir(root.join(".claude")).unwrap();
    fs::create_dir(root.join(".codex")).unwrap();
    let claude = root.join(".claude/settings.json");
    let codex = root.join(".codex/hooks.json");
    let original = b"{\"theme\":\"dark\",\"hooks\":{\"PreToolUse\":[{\"matcher\":\"Read\",\"hooks\":[{\"type\":\"command\",\"command\":\"other-hook\"}]}]}}";
    fs::write(&claude, original).unwrap();
    let args = ["init", "--auto", "--home", root.to_str().unwrap()];
    let output = run(root, &[args.as_slice(), &["--dry-run"]].concat(), None);
    success(&output);
    assert_eq!(fs::read(&claude).unwrap(), original);
    assert!(!codex.exists());
    success(&run(root, &args, None));
    let installed = fs::read(&claude).unwrap();
    let value: Value = serde_json::from_slice(&installed).unwrap();
    assert_eq!(value["theme"], "dark");
    assert_eq!(
        value["hooks"]["PreToolUse"][0]["hooks"][0]["command"],
        "other-hook"
    );
    let value: Value = serde_json::from_slice(&fs::read(&codex).unwrap()).unwrap();
    assert_eq!(value["hooks"]["PreToolUse"][0]["matcher"], "Bash");
    let backup = fs::read_dir(root.join(".claude"))
        .unwrap()
        .flatten()
        .find(|f| f.file_name().to_string_lossy().contains("stk-backup"))
        .unwrap();
    assert_eq!(fs::read(backup.path()).unwrap(), original);
    success(&run(root, &args, None));
    assert_eq!(fs::read(&claude).unwrap(), installed);
    assert_eq!(fs::read_dir(root.join(".claude")).unwrap().count(), 2);
    success(&run(
        root,
        &[args.as_slice(), &["--uninstall"]].concat(),
        None,
    ));
    let value: Value = serde_json::from_slice(&fs::read(&claude).unwrap()).unwrap();
    assert_eq!(value["hooks"]["PreToolUse"].as_array().unwrap().len(), 1);
    assert_eq!(value["theme"], "dark");
}

#[test]
fn malformed_settings_prevent_all_writes() {
    let dir = testutil::tempdir().unwrap();
    let root = dir.path();
    fs::create_dir(root.join(".claude")).unwrap();
    fs::create_dir(root.join(".codex")).unwrap();
    fs::write(root.join(".codex/hooks.json"), "{bad").unwrap();
    let output = run(
        root,
        &["init", "--auto", "--home", root.to_str().unwrap()],
        None,
    );
    assert!(!output.status.success());
    assert!(!root.join(".claude/settings.json").exists());
    assert_eq!(
        fs::read_to_string(root.join(".codex/hooks.json")).unwrap(),
        "{bad"
    );
}
