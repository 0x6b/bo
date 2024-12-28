use std::path::PathBuf;

use bo::BookmarkManager;
use clap::Parser;

#[derive(Debug, Parser)]
#[clap(version)]
pub struct Args {
    /// The name of the bookmark to open. If not provided, select from a list of available
    /// bookmarks.
    #[arg()]
    pub name: Option<String>,

    /// Path to the configuration file. Defaults to $XDG_CONFIG_HOME/bo/config.toml.
    #[arg(short, long)]
    pub config: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let Args { name, config } = Args::parse();
    let manager = BookmarkManager::from(config).await?;

    match name {
        Some(name) => manager.open(&name),
        None => manager.open_prompt(),
    }
}
