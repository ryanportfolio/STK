mod config;
#[cfg(test)]
mod testutil;
mod gain;
mod hook;
mod outline;
mod store;

use clap::{Parser, Subcommand};
use config::Config;
use store::Store;

#[derive(Parser)]
#[command(name = "stk", version, about = "Session Token Killer: clamps oversized Read tool results via a Claude Code PreToolUse hook")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Hook entry points (stdin JSON -> decision JSON)
    Hook {
        #[command(subcommand)]
        target: HookTarget,
    },
    /// Print the outline for a file (manual/debug)
    Outline { path: String },
    /// Aggregate stats: clamps, dup hits, bytes avoided, est. tokens
    Gain {
        /// Emit machine-readable JSON (totals + per-day series) for dashboards
        #[arg(long)]
        json: bool,
    },
    /// Print the settings.json hook snippet + install instructions (does NOT edit settings)
    Init,
    /// Print active config
    Config,
}

#[derive(Subcommand)]
enum HookTarget {
    /// Claude Code PreToolUse hook for the Read tool
    Claude,
}

const INIT_SNIPPET: &str = r#"stk init: install instructions
================================

stk never edits your settings. Add this to your Claude Code settings.json
(user: %USERPROFILE%\.claude\settings.json, or project: .claude/settings.json),
merging into any existing "hooks" object:

{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Read",
        "hooks": [
          { "type": "command", "command": "stk hook claude" }
        ]
      }
    ]
  }
}

Requirements:
  - `stk` must be on PATH (cargo install --path . puts it in %USERPROFILE%\.cargo\bin).
  - Verify with: echo {} | stk hook claude   (should print nothing, exit 0)

Tune behavior via %APPDATA%\stk\config.toml (all keys optional):
  clamp_threshold = 16384      # bytes
  outline_max_lines = 80
  dedup = true
  exclude = ["*.lock"]
"#;

fn main() {
    let cli = Cli::parse();
    let code = match cli.command {
        Command::Hook { target: HookTarget::Claude } => hook::run(),
        Command::Outline { path } => {
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    let cfg = Config::load();
                    let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                    println!(
                        "{}",
                        outline::generate(&path, &content, size, cfg.clamp_threshold, cfg.outline_max_lines)
                    );
                    0
                }
                Err(e) => {
                    eprintln!("stk outline: cannot read {path}: {e}");
                    1
                }
            }
        }
        Command::Gain { json } => {
            let store = Store::new(config::store_root());
            if json {
                println!("{}", gain::report_json(&store));
            } else {
                println!("{}", gain::report(&store));
            }
            0
        }
        Command::Init => {
            println!("{INIT_SNIPPET}");
            0
        }
        Command::Config => {
            let cfg = Config::load();
            let path = Config::path()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "<unknown>".into());
            let exists = Config::path().map(|p| p.exists()).unwrap_or(false);
            println!("config file: {path} ({})", if exists { "present" } else { "absent, using defaults" });
            println!("store root:  {}", config::store_root().display());
            println!();
            println!("clamp_threshold   = {}", cfg.clamp_threshold);
            println!("outline_max_lines = {}", cfg.outline_max_lines);
            println!("dedup             = {}", cfg.dedup);
            println!("exclude           = {:?}", cfg.exclude);
            let invalid = cfg.invalid_excludes();
            if !invalid.is_empty() {
                println!("WARNING: invalid exclude patterns (ignored): {invalid:?}");
            }
            0
        }
    };
    std::process::exit(code);
}
