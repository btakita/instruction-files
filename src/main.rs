use anyhow::Result;
use clap::{Parser, Subcommand};
use instruction_files::{AuditConfig, check_library_context_policy};
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

        /// Ontology directory for validating [term:Name] annotations (requires ontology feature)
        #[arg(long)]
        ontology_dir: Option<PathBuf>,
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

    /// Check that every library AGENTS.md has a Library Context Policy section
    CheckPolicy {
        /// Workspace root containing src/ with library submodules
        #[arg(short, long)]
        root: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Audit {
            root,
            broad: _,
            ontology_dir,
        } => {
            let config = AuditConfig::agent_doc();
            #[cfg(feature = "ontology")]
            {
                let _ = &ontology_dir;
                instruction_files::run(&config, root.as_deref(), ontology_dir.as_deref())?;
            }
            #[cfg(not(feature = "ontology"))]
            {
                let _ = &ontology_dir;
                instruction_files::run(&config, root.as_deref())?;
            }
        }
        Commands::Init { root } => {
            let root = root.unwrap_or_else(|| PathBuf::from("."));
            let written = instruction_files::init(&root)?;
            if written.is_empty() {
                eprintln!("Already initialized.");
            } else {
                for path in &written {
                    eprintln!("  Installed: {}", path.display());
                }
                eprintln!("Initialized {} item(s).", written.len());
            }
        }
        Commands::CheckPolicy { root } => {
            let root = root
                .map(|r| std::fs::canonicalize(&r).unwrap_or(r))
                .unwrap_or_else(|| std::env::current_dir().unwrap());
            println!("Checking library context policy...\n");

            // Scan for AGENTS.md files in src/ subdirectories (non-recursive)
            // and also check nested library AGENTS.md one level deeper
            let mut issues = Vec::new();
            let src_dir = root.join("src");
            if src_dir.is_dir() {
                let mut agents_files = Vec::new();
                // Collect AGENTS.md from src/*/AGENTS.md and src/*/*/AGENTS.md
                for depth_pattern in &["src/*/AGENTS.md", "src/*/*/AGENTS.md"] {
                    let pattern = root.join(depth_pattern).to_string_lossy().to_string();
                    for path in glob::glob(&pattern).unwrap_or_else(|_| panic!("invalid glob: {}", pattern)).flatten() {
                        agents_files.push(path);
                    }
                }
                for path in agents_files {
                    let Ok(rel) = path.strip_prefix(&root) else {
                        continue;
                    };
                    let rel_str = rel.to_string_lossy().to_string();
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        issues.extend(check_library_context_policy(&rel_str, &content, &root));
                    }
                }
            }

            if issues.is_empty() {
                println!("All library AGENTS.md files have valid Library Context Policy sections \u{2713}");
            } else {
                for issue in &issues {
                    let marker = if issue.warning { "\u{26a0}" } else { "\u{2717}" };
                    let mut loc = format!("  {}", issue.file);
                    if issue.line > 0 {
                        loc.push_str(&format!(":{}", issue.line));
                    }
                    println!("{:<50} {} {}", loc, marker, issue.message);
                }
                println!("\nFound {} issue(s)", issues.len());
                std::process::exit(1);
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
