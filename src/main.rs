use anyhow::Result;
use clap::Parser;
use helix::cli::commands::Cli;
use log::info;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    let cli = Cli::parse();
    info!("Helix block-level backup engine v{}", env!("CARGO_PKG_VERSION"));

    match cli.command {
        Some(cmd) => helix::cli::commands::dispatch(cmd).await?,
        None => {
            println!("Helix - Enterprise-Grade Block-Level Backup Engine");
            println!("Use --help for available commands");
        }
    }

    Ok(())
}
