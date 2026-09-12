//! Merge only STK-owned hook handlers; retain every unrelated setting.
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy)]
pub enum Client {
    Claude,
    Codex,
}
impl Client {
    pub fn name(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
        }
    }
    fn matcher(self) -> &'static str {
        match self {
            Self::Claude => "Read",
            Self::Codex => "Bash",
        }
    }
    pub fn config_path(self, home: &Path, honor_env: bool) -> PathBuf {
        let (variable, folder, file) = match self {
            Self::Claude => ("CLAUDE_CONFIG_DIR", ".claude", "settings.json"),
            Self::Codex => ("CODEX_HOME", ".codex", "hooks.json"),
        };
        let root = if honor_env {
            std::env::var_os(variable)
                .filter(|v| !v.is_empty())
                .map(PathBuf::from)
        } else {
            None
        };
        root.unwrap_or_else(|| home.join(folder)).join(file)
    }
}

fn owns(handler: &Value, client: Client) -> bool {
    if handler["type"] != "command" {
        return false;
    }
    let Some(command) = handler["command"].as_str() else {
        return false;
    };
    let suffix = format!(" hook {}", client.name());
    let Some(exe) = command.trim().strip_suffix(&suffix) else {
        return false;
    };
    let exe = exe.trim_matches('"').replace('\\', "/");
    matches!(exe.rsplit('/').next(), Some("stk" | "stk.exe"))
}

pub fn merge(
    mut value: Value,
    client: Client,
    executable: &Path,
    uninstall: bool,
) -> Result<Value, String> {
    let object = value
        .as_object_mut()
        .ok_or("settings must be a JSON object")?;
    if uninstall && !object.contains_key("hooks") {
        return Ok(value);
    }
    let hooks = object
        .entry("hooks")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .ok_or("hooks must be an object")?;
    if uninstall && !hooks.contains_key("PreToolUse") {
        return Ok(value);
    }
    let groups = hooks
        .entry("PreToolUse")
        .or_insert_with(|| json!([]))
        .as_array_mut()
        .ok_or("PreToolUse must be an array")?;
    let mut removed_groups = Vec::new();
    for (index, group) in groups.iter_mut().enumerate() {
        let handlers = group
            .get_mut("hooks")
            .and_then(Value::as_array_mut)
            .ok_or("each PreToolUse group must contain a hooks array")?;
        let had_handlers = !handlers.is_empty();
        handlers.retain(|h| !owns(h, client));
        if had_handlers && handlers.is_empty() {
            removed_groups.push(index);
        }
    }
    for index in removed_groups.into_iter().rev() {
        groups.remove(index);
    }
    if !uninstall {
        let executable = executable
            .to_str()
            .ok_or("executable path must be UTF-8")?
            .replace('\\', "/");
        if executable.chars().any(|c| "\"$`\n\r%".contains(c)) {
            return Err("executable path contains unsupported shell characters".into());
        }
        let command = format!("\"{executable}\" hook {}", client.name());
        // Explicit Windows command is required by some Codex hook runners.
        // PowerShell's call operator also handles executable paths with spaces.
        let windows_command = format!(
            "powershell.exe -NoProfile -Command \"& '{}' hook {}\"",
            executable.replace('\'', "''"),
            client.name()
        );
        groups.push(json!({
            "matcher": client.matcher(),
            "hooks": [{
                "type": "command", "timeout": 5,
                "command": command, "commandWindows": windows_command
            }]
        }));
    }
    Ok(value)
}

pub fn run(
    home: Option<PathBuf>,
    auto: bool,
    claude: bool,
    codex: bool,
    dry_run: bool,
    uninstall: bool,
) -> Result<(), String> {
    let honor_env = home.is_none();
    let home = home
        .or_else(dirs::home_dir)
        .ok_or("cannot locate home directory; pass --home")?;
    let executable = std::env::current_exe().map_err(|e| e.to_string())?;
    let mut plans = Vec::new();
    for (client, selected) in [(Client::Claude, claude), (Client::Codex, codex)] {
        let path = client.config_path(&home, honor_env);
        if !(selected || auto && path.parent().is_some_and(Path::is_dir)) {
            continue;
        }
        let old = match fs::read(&path) {
            Ok(bytes) => Some(bytes),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(format!("{}: {e}", path.display())),
        };
        if uninstall && old.is_none() {
            continue;
        }
        let value = match &old {
            Some(bytes) => {
                serde_json::from_slice(bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(bytes))
                    .map_err(|e| format!("{}: {e}; left unchanged", path.display()))?
            }
            None => json!({}),
        };
        let next = merge(value.clone(), client, &executable, uninstall)
            .map_err(|e| format!("{}: {e}", path.display()))?;
        let changed = value != next;
        let text = serde_json::to_string_pretty(&next).map_err(|e| e.to_string())? + "\n";
        plans.push((client, path, old, text, changed));
    }
    if plans.is_empty() {
        return Err("no client settings directories detected; use --claude or --codex".into());
    }
    // Validate all selected settings before changing any file.
    for (client, path, old, text, changed) in plans {
        if dry_run {
            println!("{}: {}\n{text}", client.name(), path.display());
        } else if changed {
            write_config(&path, old.as_deref(), text.as_bytes())?;
            println!(
                "{}: {} {}",
                client.name(),
                if uninstall {
                    "removed STK hooks from"
                } else {
                    "configured"
                },
                path.display()
            );
        } else {
            println!("{}: already up to date ({})", client.name(), path.display());
        }
        if matches!(client, Client::Codex) && !uninstall {
            println!("Codex: restart the client, then review and trust STK in /hooks. Registration does not grant hook trust.");
        }
    }
    Ok(())
}

fn write_config(path: &Path, old: Option<&[u8]>, next: &[u8]) -> Result<(), String> {
    use std::io::Write;
    // Refuse to overwrite a concurrent settings edit.
    let current = match fs::read(path) {
        Ok(b) => Some(b),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(e.to_string()),
    };
    if current.as_deref() != old {
        return Err(format!("{} changed during setup; retry", path.display()));
    }
    fs::create_dir_all(path.parent().ok_or("missing parent directory")?)
        .map_err(|e| e.to_string())?;
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    if let Some(old) = old {
        let backup = path.with_extension(format!("json.stk-backup-{nonce}"));
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&backup)
            .map_err(|e| e.to_string())?;
        file.write_all(old)
            .and_then(|_| file.sync_all())
            .map_err(|e| e.to_string())?;
        println!("Backup: {}", backup.display());
    }
    let temp = path.with_extension(format!("json.stk-tmp-{nonce}"));
    let result = (|| -> std::io::Result<()> {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)?;
        file.write_all(next)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temp, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result.map_err(|e| format!("{}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_others_upgrades_legacy_and_is_idempotent() {
        let original = json!({"theme":"dark","hooks":{"Stop":[{"hooks":[]}],"PreToolUse":[
            {"matcher":"Read", "hooks":[{"type":"command","command":"stk hook claude"},{"type":"command","command":"other-hook"}]}
        ]}});
        let exe = Path::new("/a path/stk");
        let next = merge(original.clone(), Client::Claude, exe, false).unwrap();
        assert_eq!(next["theme"], "dark");
        assert_eq!(next["hooks"]["Stop"], original["hooks"]["Stop"]);
        assert_eq!(
            next["hooks"]["PreToolUse"][0]["hooks"][0]["command"],
            "other-hook"
        );
        assert_eq!(
            merge(next.clone(), Client::Claude, exe, false).unwrap(),
            next
        );
        let removed = merge(next, Client::Claude, exe, true).unwrap();
        assert_eq!(removed["hooks"]["PreToolUse"].as_array().unwrap().len(), 1);
        assert_eq!(
            removed["hooks"]["PreToolUse"][0]["hooks"][0]["command"],
            "other-hook"
        );
    }
    #[test]
    fn invalid_structures_rejected_and_unrelated_commands_preserved() {
        for value in [
            json!([]),
            json!({"hooks":null}),
            json!({"hooks":{"PreToolUse":{}}}),
            json!({"hooks":{"PreToolUse":[{}]}}),
        ] {
            assert!(merge(value, Client::Codex, Path::new("/stk"), false).is_err());
        }
        assert!(!owns(
            &json!({"type":"command","command":"echo stk hook codex"}),
            Client::Codex
        ));
        assert!(!owns(
            &json!({"type":"command","command":"stk hook claude"}),
            Client::Codex
        ));
    }
}
