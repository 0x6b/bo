use std::path::PathBuf;

use bo::BookmarkManager;
use clap::Parser;

#[derive(Debug, Parser)]
#[clap(version)]
pub struct Args {
    /// The name of the bookmark to open. If not provided, a list of available bookmarks will be
    /// shown. If multiple strings are given, the first one is used as the bookmark name, and the
    /// remaining strings are used as arguments for the bookmark, which will be replaced in the URL
    /// `{query}`.
    #[arg()]
    pub args: Option<Vec<String>>,

    /// Path to the configuration file. Defaults to $XDG_CONFIG_HOME/bo/config.toml.
    #[arg(short, long)]
    pub config: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let Args { args, config } = Args::parse();
    let manager = BookmarkManager::from(config).await?;

    match args {
        Some(args) if args.len() == 1 => manager.open(args.first().unwrap()),
        Some(args) if args.len() > 1 => manager.search(&args),
        _ => manager.open_prompt(),
    }
}
