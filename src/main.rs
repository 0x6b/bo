use std::{env, fmt, fmt::Display, path::PathBuf, process, str::FromStr};

use bo::BookmarkManager;
use clap::Parser;
use tokio::{
    fs::File,
    io::{AsyncWriteExt, BufWriter},
};

#[derive(Debug, Parser)]
#[clap(version)]
pub struct Args {
    /// The name of the bookmark to open. If not provided, a list of available bookmarks will be
    /// shown. If multiple strings are given, the first one is used as the bookmark name, and the
    /// remaining strings are used as arguments for the bookmark, which will be replaced in the URL
    /// `{query}`.
    #[arg()]
    pub args: Option<Vec<String>>,

    /// Path to the configuration file. Defaults to $XDG_CONFIG_HOME/bo/config.yaml.
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    #[clap(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Parser)]
pub enum Command {
    /// Add a new bookmark.
    Add {
        /// The name of the bookmark.
        name: String,

        /// The URL of the bookmark.
        url: String,

        /// The browser to use. If not provided, the default browser will be used.
        #[arg(short, long)]
        browser: Option<String>,
    },

    /// Edit the configuration file with $EDITOR.
    Edit,

    /// Generate a shell completion script. At the moment, only `fish` is supported.
    GenerateCompletion {
        /// The shell to generate completion scripts for.
        #[arg(short, long, default_value = "fish")]
        shell: Shell,

        /// The path to write the completion script to.
        #[arg(short, long, default_value = "~/.config/fish/completions/bo.fish")]
        path: String,
    },
}

#[derive(Debug, Clone)]
pub enum Shell {
    Fish,
}

impl Display for Shell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fish => write!(f, "fish"),
        }
    }
}

impl FromStr for Shell {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        match s {
            "fish" => Ok(Self::Fish),
            _ => Err(anyhow::anyhow!("Unsupported shell: {s}")),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let Args { args, config, command } = Args::parse();
    let manager = BookmarkManager::from(config).await?;

    match command {
        Some(Command::Add { name, url, browser }) => {
            manager.add(name, url, browser.as_deref()).await?;
            Ok(())
        }
        Some(Command::Edit) => {
            match process::Command::new(env::var("EDITOR").unwrap_or_else(|_| "vi".to_string()))
                .arg(&manager.path)
                .spawn()
            {
                Ok(mut child) => child.wait().map(|_| ()).map_err(Into::into),
                Err(e) => anyhow::bail!("Failed to open editor: {e}"),
            }
        }
        Some(Command::GenerateCompletion { shell: _, path }) => {
            generate_fish_completion(&manager, &path).await
        }
        None => match args {
            Some(args) if args.len() == 1 => manager.open(args.first().unwrap()),
            Some(args) if args.len() > 1 => manager.search(&args),
            _ => manager.open_prompt(),
        },
    }
}

async fn generate_fish_completion(manager: &BookmarkManager, path: &str) -> anyhow::Result<()> {
    let file = PathBuf::from(shellexpand::tilde(&path).to_string());
    let mut out = BufWriter::new(File::create(&file).await?);

    out.write_all(b"# fish shell completions for bo command\n").await?;
    out.write_all(b"complete -c bo -f -n \"not __fish_seen_subcommand_from completion\"\n")
        .await?;
    out.write_all(
        b"complete -c bo -f -n \"not __fish_seen_subcommand_from completion\" -a generate-completion -d \"command: Generate shell completion script\"\n",
    )
        .await?;

    for (name, url_config) in manager.bookmarks.iter() {
        let url = simplify_url(&url_config.url);
        out.write_all(
            format!(
                "complete -c bo -f -n \"not __fish_seen_subcommand_from completion\" -a \"{name}\" -d \"{url}\"\n",
            )
                .as_bytes(),
        )
            .await?;
    }

    if let Some(aliases) = manager.aliases.as_ref() {
        for (alias, name) in aliases.iter() {
            if let Some(url_config) = manager.bookmarks.get(name) {
                let url = simplify_url(&url_config.url);
                out.write_all(
                    format!(
                        "complete -c bo -f -n \"not __fish_seen_subcommand_from completion\" -a \"{alias}\" -d \"{url}\"\n",
                    )
                        .as_bytes(),
                )
                    .await?
            } else {
                eprintln!("Warning: alias '{alias}' points to a non-existent bookmark: '{name}'. Skipping.");
            }
        }
    }

    out.flush().await?;
    println!("Auto completion file updated: {}", file.display());

    Ok(())
}

fn simplify_url(url: &str) -> String {
    url.replace("https://", "").replace("http://", "").replace("www.", "")
}
