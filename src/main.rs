use anyhow::Result;
use clap::{Parser, Subcommand};
use instruction_files::AuditConfig;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "instruction-files", about = "Audit and validate AI agent instruction files")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the full audit suite across all discovered instruction files
    Audit {
        /// Project root (default: auto-detect from CWD)
        #[arg(short, long)]
        root: Option<PathBuf>,

        /// Use broad config (many languages, many root markers)
        #[arg(long, default_value_t = true)]
        broad: bool,
    },

    /// Initialize .agent/runbooks/ with bundled defaults
    Init {
        /// Project root (default: CWD)
        #[arg(short, long)]
        root: Option<PathBuf>,
    },

    /// List all discovered instruction files
    List {
        /// Project root (default: auto-detect from CWD)
        #[arg(short, long)]
        root: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Audit { root, broad: _ } => {
            let config = AuditConfig::agent_doc();
            instruction_files::run(&config, root.as_deref())?;
        }
        Commands::Init { root } => {
            let root = root.unwrap_or_else(|| PathBuf::from("."));
            let written = instruction_files::init_runbooks(&root)?;
            if written > 0 {
                eprintln!("Initialized {written} runbook(s) in .agent/runbooks/");
            } else {
                eprintln!("Runbooks already initialized.");
            }
        }
        Commands::List { root } => {
            let config = AuditConfig::agent_doc();
            let project_root = match root {
                Some(r) => r,
                None => instruction_files::find_root(&config),
            };
            let files = instruction_files::find_instruction_files(&project_root, &config);
            if files.is_empty() {
                println!("No instruction files found.");
            } else {
                for f in &files {
                    let rel = f.strip_prefix(&project_root).unwrap_or(f);
                    println!("  {}", rel.display());
                }
                println!("\n{} file(s) found.", files.len());
            }
        }
    }

    Ok(())
}
