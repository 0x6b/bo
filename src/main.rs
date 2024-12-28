use std::{collections::BTreeMap, ops::Deref, path::PathBuf, sync::Arc};

use anyhow::{anyhow, Result};
use clap::Parser;
use regex::Regex;
use serde::{Deserialize, Serialize};
use skim::{
    options::SkimOptionsBuilder, prelude::unbounded, Skim, SkimItemReceiver, SkimItemSender,
};
use tokio::fs::read_to_string;
use toml::from_str;
use xdg::BaseDirectories;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Default browser to use.
    pub default_browser: String,

    /// Bookmark aliases
    pub aliases: Option<BTreeMap<String, String>>,

    /// Bookmark name to URL mappings.
    pub bookmarks: BTreeMap<String, UrlConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlConfig {
    pub url: String,
    pub browser: Option<String>,
}

/// A bookmark manager.
#[derive(Debug)]
pub struct BookmarkManager {
    /// The configuration.
    config: Config,
}

impl Deref for BookmarkManager {
    type Target = Config;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

impl BookmarkManager {
    /// Create a new `BookmarkManager` instance.
    ///
    /// # Arguments
    ///
    /// - `config` - Path to the configuration file.
    pub async fn from(config_path: Option<PathBuf>) -> Result<Self> {
        let (_path, config) = Self::parse_config(config_path).await?;
        Ok(Self { config })
    }

    async fn parse_config(path: Option<PathBuf>) -> Result<(PathBuf, Config)> {
        let path = path.unwrap_or_else(|| {
            BaseDirectories::with_prefix("bo")
                .unwrap()
                .place_config_file("config.toml")
                .unwrap()
        });
        let config = from_str::<Config>(&read_to_string(&path).await?)?;

        Ok((path, config))
    }

    /// Open the URL matching the given name.
    pub fn open(&self, name: &str) -> Result<()> {
        let url_config = self
            .get_url_config(name)
            .ok_or_else(|| anyhow!("Bookmark not found: {name}"))?;

        let browser = match &url_config.browser {
            Some(browser) => browser,
            None => self.default_browser.as_str(),
        };

        open::with(&url_config.url, browser).map_err(Into::into)
    }

    /// Open an interactive prompt to select a bookmark to open.
    pub fn open_prompt(&self) -> Result<()> {
        let options = SkimOptionsBuilder::default()
            .height(String::from("5"))
            .multi(false)
            .build()?;

        let (tx_item, rx_item): (SkimItemSender, SkimItemReceiver) = unbounded();
        self.bookmarks.iter().for_each(|(name, url_config)| {
            let browser = match &url_config.browser {
                Some(browser) => browser,
                None => self.default_browser.as_str(),
            };

            let _ = tx_item.send(Arc::new(format!("{name}: {} (in {browser})", url_config.url)));
        });
        drop(tx_item);

        match Skim::run_with(&options, Some(rx_item)) {
            Some(out) if out.is_abort => Ok(()),
            Some(out) if !out.selected_items.is_empty() => {
                self.open(&out.selected_items.first().unwrap().text())
            }
            _ => Ok(()),
        }
    }

    fn get_url_config(&self, name: &str) -> Option<&UrlConfig> {
        let name = Regex::new(r":\shttp.+\s\(in\s(\w+)\)$").unwrap().replace(name, "");
        let name = name.trim();
        self.get_url_config_from_name(name)
            .or_else(|| self.get_url_config_from_alias(name))
    }

    fn get_url_config_from_name(&self, name: &str) -> Option<&UrlConfig> {
        self.bookmarks.get(name)
    }

    fn get_url_config_from_alias(&self, alias: &str) -> Option<&UrlConfig> {
        self.aliases
            .as_ref()
            .and_then(|aliases| aliases.get(alias))
            .and_then(|name| self.bookmarks.get(name))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let Args { name: bookmark_name, config } = Args::parse();
    let manager = BookmarkManager::from(config).await?;

    match bookmark_name {
        Some(bookmark_name) => manager.open(&bookmark_name),
        None => manager.open_prompt(),
    }
}
