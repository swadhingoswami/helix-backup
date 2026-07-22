use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;

#[derive(Parser, Debug)]
#[command(name = "helix")]
#[command(about = "Enterprise-Grade Block-Level Backup Engine", long_about = None)]
#[command(version, propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(
        global = true,
        short = 'v',
        long = "verbose",
        help = "Increase verbosity"
    )]
    pub verbose: bool,

    #[arg(
        global = true,
        short = 'c',
        long = "config",
        help = "Path to configuration file"
    )]
    pub config: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Perform a full backup of target device")]
    Full {
        #[arg(help = "Source device path (e.g. /dev/sda)")]
        source: String,

        #[arg(short = 'd', long = "dest", help = "Backup destination path")]
        dest: String,

        #[arg(long = "label", help = "Backup label/name")]
        label: Option<String>,

        #[arg(
            long = "block-size",
            default_value = "4096",
            help = "Block size in bytes"
        )]
        block_size: u32,
    },

    #[command(about = "Perform an incremental backup")]
    Incremental {
        #[arg(help = "Source device path")]
        source: String,

        #[arg(short = 'd', long = "dest", help = "Backup repository path")]
        dest: String,

        #[arg(long = "label", help = "Backup label")]
        label: Option<String>,

        #[arg(
            long = "block-size",
            default_value = "4096",
            help = "Block size in bytes"
        )]
        block_size: u32,
    },

    #[command(about = "Restore from a backup")]
    Restore {
        #[arg(help = "Source repository path")]
        source: String,

        #[arg(help = "Target device path for restore")]
        target: String,

        #[arg(
            short = 'p',
            long = "point",
            help = "Restore point (latest, full, or incremental ID)"
        )]
        point: Option<String>,
    },

    #[command(about = "List available backups")]
    List {
        #[arg(help = "Repository path")]
        path: String,

        #[arg(long = "json", help = "Output in JSON format")]
        json: bool,
    },

    #[command(about = "Validate repository integrity")]
    Check {
        #[arg(help = "Repository path")]
        path: String,

        #[arg(long = "repair", help = "Attempt to repair issues")]
        repair: bool,
    },

    #[command(about = "Initialize a new backup repository")]
    Init {
        #[arg(help = "Repository path")]
        path: String,

        #[arg(short = 'k', long = "key", help = "Encryption key file")]
        key: Option<String>,

        #[arg(
            long = "compression-level",
            default_value = "3",
            help = "ZSTD compression level (1-22)"
        )]
        compression_level: i32,
    },

    #[command(about = "Display configuration information")]
    Config {
        #[command(subcommand)]
        action: Option<ConfigCommands>,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    #[command(about = "Show current configuration")]
    Show,
    #[command(about = "Validate configuration file")]
    Validate {
        #[arg(help = "Path to configuration file")]
        path: String,
    },
}

pub async fn dispatch(command: Commands) -> Result<()> {
    match command {
        Commands::Full {
            source,
            dest,
            label,
            block_size,
        } => {
            println!(
                "{} full backup from {} to {}",
                "[Starting]".green(),
                source,
                dest
            );
            let engine = crate::backup::engine::BackupEngine::new(block_size)?;
            engine
                .run_full_backup(&source, &dest, label.as_deref())
                .await?;
            println!("{} Full backup completed successfully", "[Done]".green());
        }
        Commands::Incremental {
            source,
            dest,
            label,
            block_size,
        } => {
            println!(
                "{} incremental backup from {} to {}",
                "[Starting]".green(),
                source,
                dest
            );
            let engine = crate::backup::engine::BackupEngine::new(block_size)?;
            engine
                .run_incremental_backup(&source, &dest, label.as_deref())
                .await?;
            println!(
                "{} Incremental backup completed successfully",
                "[Done]".green()
            );
        }
        Commands::Restore {
            source,
            target,
            point,
        } => {
            println!(
                "{} restoring from {} to {}",
                "[Starting]".green(),
                source,
                target
            );
            let engine = crate::restore::engine::RestoreEngine::new()?;
            engine
                .run_restore(&source, &target, point.as_deref())
                .await?;
            println!("{} Restore completed successfully", "[Done]".green());
        }
        Commands::List { path, json } => {
            let repo = crate::repository::layout::Repository::open(&path)?;
            let backups = repo.list_backups()?;
            if json {
                println!("{}", serde_json::to_string_pretty(&backups)?);
            } else {
                println!("{} backups in {}:", backups.len(), path);
                for b in &backups {
                    println!(
                        "  {} - {} ({} blocks, {})",
                        b.id, b.timestamp, b.block_count, b.backup_type
                    );
                }
            }
        }
        Commands::Check { path, repair } => {
            let repo = crate::repository::layout::Repository::open(&path)?;
            let result = repo.validate(repair)?;
            println!(
                "Repository check: {}",
                if result.is_ok() { "PASSED" } else { "FAILED" }
            );
            for issue in result.issues() {
                println!("  {}: {}", "[WARN]".yellow(), issue);
            }
        }
        Commands::Init {
            path,
            key,
            compression_level,
        } => {
            crate::repository::layout::Repository::initialize(
                &path,
                key.as_deref(),
                compression_level,
            )?;
            println!("{} Repository initialized at {}", "[Created]".green(), path);
        }
        Commands::Config { action } => match action {
            Some(ConfigCommands::Show) => {
                let config = crate::config::loader::load_config(None)?;
                println!("{}", serde_yaml::to_string(&config)?);
            }
            Some(ConfigCommands::Validate { path }) => {
                match crate::config::validator::validate_file(&path) {
                    Ok(()) => println!("{} Configuration is valid", "[OK]".green()),
                    Err(e) => println!("{} Configuration invalid: {}", "[Error]".red(), e),
                }
            }
            None => {
                println!("Use --help for config subcommands");
            }
        },
    }
    Ok(())
}
