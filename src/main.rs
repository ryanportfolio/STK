mod codex;
mod config;
mod gain;
mod hook;
mod install;
mod outline;
mod read;
mod store;
#[cfg(test)]
mod testutil;

use clap::{Parser, Subcommand};
use config::Config;
use store::Store;

#[derive(Parser)]
#[command(
    name = "stk",
    version,
    about = "Session Token Killer: compact file reads for Claude Code and Codex"
)]
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
    /// Read a file; outline large files unless an explicit range is requested
    Read {
        path: std::path::PathBuf,
        /// First line (1-based); bypasses outlining
        #[arg(long)]
        offset: Option<usize>,
        /// Maximum number of lines; bypasses outlining
        #[arg(long)]
        limit: Option<usize>,
    },
    /// Aggregate stats: clamps, dup hits, bytes avoided, est. tokens
    Gain {
        /// Emit machine-readable JSON (totals + per-day series) for dashboards
        #[arg(long)]
        json: bool,
    },
    /// Print legacy instructions, or configure clients with explicit selection flags
    Init {
        /// Configure clients whose settings directories exist
        #[arg(long, conflicts_with_all = ["claude", "codex"])]
        auto: bool,
        /// Configure Claude Code
        #[arg(long)]
        claude: bool,
        /// Configure Codex
        #[arg(long)]
        codex: bool,
        /// Print proposed settings without writing them
        #[arg(long)]
        dry_run: bool,
        /// Remove only STK hook handlers
        #[arg(long)]
        uninstall: bool,
        /// Alternate user home (ignores client directory environment overrides)
        #[arg(long)]
        home: Option<std::path::PathBuf>,
    },
    /// Print active config
    Config,
}

#[derive(Subcommand)]
enum HookTarget {
    /// Claude Code PreToolUse hook for the Read tool
    Claude,
    /// Codex PreToolUse hook for supported whole-file shell reads
    Codex,
}

const INIT_SNIPPET: &str = r#"stk init: install instructions
================================

This legacy snippet does not edit settings. Add it to your Claude Code settings.json
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
        Command::Hook { target } => hook::run(matches!(target, HookTarget::Codex)),
        Command::Read {
            path,
            offset,
            limit,
        } => {
            match read::read_to(
                &path,
                offset,
                limit,
                &Config::load(),
                &mut std::io::stdout().lock(),
            ) {
                Ok(()) => 0,
                Err(e) => {
                    eprintln!("stk read: {}: {e}", path.display());
                    1
                }
            }
        }
        Command::Outline { path } => match std::fs::read_to_string(&path) {
            Ok(content) => {
                let cfg = Config::load();
                let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                println!(
                    "{}",
                    outline::generate(
                        &path,
                        &content,
                        size,
                        cfg.clamp_threshold,
                        cfg.outline_max_lines
                    )
                );
                0
            }
            Err(e) => {
                eprintln!("stk outline: cannot read {path}: {e}");
                1
            }
        },
        Command::Gain { json } => {
            let store = Store::new(config::store_root());
            if json {
                println!("{}", gain::report_json(&store));
            } else {
                println!("{}", gain::report(&store));
            }
            0
        }
        Command::Init {
            auto,
            claude,
            codex,
            dry_run,
            uninstall,
            home,
        } => {
            if !auto && !claude && !codex {
                if dry_run || uninstall || home.is_some() {
                    eprintln!("stk init: select --auto, --claude, or --codex");
                    1
                } else {
                    println!("{INIT_SNIPPET}\nAutomatic setup: stk init --auto\nChoose a client: stk init --claude / stk init --codex\nPreview: add --dry-run. Remove STK hooks: add --uninstall.");
                    0
                }
            } else {
                match install::run(home, auto, claude, codex, dry_run, uninstall) {
                    Ok(()) => 0,
                    Err(e) => {
                        eprintln!("stk init: {e}");
                        1
                    }
                }
            }
        }
        Command::Config => {
            let cfg = Config::load();
            let path = Config::path()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "<unknown>".into());
            let exists = Config::path().map(|p| p.exists()).unwrap_or(false);
            println!(
                "config file: {path} ({})",
                if exists {
                    "present"
                } else {
                    "absent, using defaults"
                }
            );
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
