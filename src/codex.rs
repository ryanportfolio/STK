//! Conservative Codex adapter. Unknown shell syntax always passes through.
use crate::{config::Config, hook};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// Deliberately not a general shell parser. Never execute a command to discover
/// its path. Reject interpolation, operators, globs, escapes and multiple reads.
pub fn read_path(command: &str) -> Option<String> {
    let command = command.trim();
    if command
        .chars()
        .any(|c| "\n\r\0;|&><$`*?[]{}()#!".contains(c))
    {
        return None;
    }
    let mut words = Vec::new();
    let mut chars = command.chars().peekable();
    while chars.peek().is_some() {
        while chars.peek().is_some_and(|c| c.is_whitespace()) {
            chars.next();
        }
        let Some(first) = chars.next() else { break };
        let mut word = String::new();
        if first == '\'' || first == '"' {
            loop {
                let c = chars.next()?;
                if c == first {
                    break;
                }
                word.push(c);
            }
            // Quoted/unquoted concatenation has different meanings in shells.
            if chars.peek().is_some_and(|c| !c.is_whitespace()) {
                return None;
            }
        } else {
            word.push(first);
            while chars.peek().is_some_and(|c| !c.is_whitespace()) {
                let c = chars.next()?;
                if c == '\'' || c == '"' {
                    return None;
                }
                word.push(c);
            }
        }
        words.push(word);
    }
    let (program, args) = words.split_first()?;
    let mut args = args.to_vec();
    match program.to_ascii_lowercase().as_str() {
        "get-content" => {
            // Accept only whole text reads. Counts, encoding, streams, wait,
            // dynamic expressions, and abbreviated parameters stay untouched.
            let mut path = None;
            let mut raw = false;
            let mut named = false;
            let mut i = 0;
            while i < args.len() {
                match args[i].to_ascii_lowercase().as_str() {
                    "-raw" if !raw => raw = true,
                    "-path" | "-literalpath" if !named && path.is_none() => {
                        named = true;
                        i += 1;
                        path = Some(args.get(i)?.clone());
                    }
                    _ if !args[i].starts_with('-') && path.is_none() => {
                        path = Some(args[i].clone())
                    }
                    _ => return None,
                }
                i += 1;
            }
            return literal_path(path?);
        }
        "rtk" if args.first().map(String::as_str) == Some("read") => {
            args.remove(0);
        }
        "cat" => {}
        _ => return None,
    }
    if args.first().map(String::as_str) == Some("--") {
        args.remove(0);
    }
    if args.len() != 1 {
        return None;
    }
    // Backslashes are escapes in POSIX shells, so don't guess their meaning.
    if args[0].contains('\\') {
        return None;
    }
    literal_path(args.remove(0))
}

fn literal_path(path: String) -> Option<String> {
    if path.is_empty() || path.starts_with(['-', '~', '@']) || path.contains(',') {
        return None;
    }
    if let Some((drive, rest)) = path.split_once(':') {
        if drive.len() != 1
            || !drive.as_bytes()[0].is_ascii_alphabetic()
            || !rest.starts_with(['/', '\\'])
            || rest.contains(':')
        {
            return None;
        }
    }
    Some(path)
}

pub fn decide(raw: &str, config: &Config, root: PathBuf) -> Option<String> {
    let event: Value = serde_json::from_str(raw).ok()?;
    if event["hook_event_name"].as_str() != Some("PreToolUse")
        || event["tool_name"].as_str() != Some("Bash")
    {
        return None;
    }
    let path = read_path(event["tool_input"]["command"].as_str()?)?;
    if config.is_excluded(&path) {
        return None;
    }
    let path = Path::new(&path);
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        let cwd = Path::new(event["cwd"].as_str()?);
        if !cwd.is_absolute() {
            return None;
        }
        cwd.join(path)
    };
    if config.is_excluded(&path.to_string_lossy()) {
        return None;
    }
    let path = path.canonicalize().ok()?;
    let path = path.to_str()?;
    // Codex gives subagents the parent session_id. Until a stable agent/context
    // identity exists, returning an outline each time avoids suppressing a map
    // another agent saw, or one lost during compaction.
    let mut config = config.clone();
    config.dedup = false;
    let adapted = json!({"tool_name":"Read", "session_id":format!("codex-{}", event["session_id"].as_str().unwrap_or("unknown")),
        "tool_input":{"file_path":path}});
    hook::decide_for(&adapted.to_string(), &config, root, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::tempdir;

    #[test]
    fn recognizes_only_literal_whole_reads() {
        for (command, expected) in [
            ("cat src/main.rs", "src/main.rs"),
            ("cat -- 'a b.rs'", "a b.rs"),
            ("rtk read \"a b.rs\"", "a b.rs"),
            (
                "Get-Content -LiteralPath 'C:\\repo\\a b.rs' -Raw",
                "C:\\repo\\a b.rs",
            ),
            ("get-content -Raw -Path a.rs", "a.rs"),
        ] {
            assert_eq!(read_path(command).as_deref(), Some(expected), "{command}");
        }
        for command in [
            "cat a b",
            "cat a | head",
            "cat a; echo x",
            "cat a && echo x",
            "cat $(pwd)/a",
            "cat $FILE",
            "cat *.rs",
            "cat ~/a",
            "cat a > b",
            "cat 'a'junk",
            "cat \"unterminated",
            "cat a\ncat b",
            "cat -n a",
            "Get-Content a -TotalCount 2",
            "Get-Content a -Tail 2",
            "Get-Content a -Encoding byte",
            "Get-Content a -Wait",
            "Get-Content a,b",
            "Get-Content @paths",
            "Get-Content a -Raw -Raw",
            "Get-Content -Path a b",
            "Get-Content -LiteralPath -Raw",
            "cat a\\ b",
            "rtk read a --max-lines 2",
            "python read.py",
            "Get-Content Env:PATH",
            "cat a # comment",
        ] {
            assert_eq!(read_path(command), None, "{command}");
        }
    }

    #[test]
    fn codex_contract_cwd_followups_and_no_cross_agent_dedup() {
        let dir = tempdir().unwrap();
        std::fs::write(
            dir.path().join("large file.rs"),
            "fn example() {}\n".repeat(3000),
        )
        .unwrap();
        let mut event = json!({"session_id":"shared", "hook_event_name":"PreToolUse", "tool_name":"Bash",
            "cwd":dir.path(), "tool_input":{"command":"Get-Content 'large file.rs'"}});
        let root = dir.path().join("store");
        for _ in 0..2 {
            let output: Value = serde_json::from_str(
                &decide(&event.to_string(), &Config::default(), root.clone()).unwrap(),
            )
            .unwrap();
            assert_eq!(output["hookSpecificOutput"]["permissionDecision"], "deny");
            let reason = output["hookSpecificOutput"]["permissionDecisionReason"]
                .as_str()
                .unwrap();
            assert!(reason.contains("stk clamp:"));
            assert!(reason.contains("stk read"));
            assert!(!reason.contains("Read(file_path"));
        }
        event["tool_input"]["command"] = json!("Get-Content 'large file.rs' -TotalCount 3");
        assert!(decide(&event.to_string(), &Config::default(), root.clone()).is_none());
        event["tool_input"]["command"] = json!("cat missing.rs");
        assert!(decide(&event.to_string(), &Config::default(), root.clone()).is_none());
        for raw in ["", "{}", "garbage", "null"] {
            assert!(decide(raw, &Config::default(), root.clone()).is_none());
        }
    }
}
