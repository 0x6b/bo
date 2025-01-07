use std::{ops::Deref, path::PathBuf, sync::Arc};

use skim::{
    options::SkimOptionsBuilder, prelude::unbounded, Skim, SkimItemReceiver, SkimItemSender,
};

use crate::{Config, UrlConfig};

/// A bookmark manager.
#[derive(Debug)]
pub struct BookmarkManager {
    /// Resolved path to the configuration file.
    pub path: PathBuf,

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
    pub async fn from(config_path: Option<PathBuf>) -> anyhow::Result<Self> {
        let (path, config) = Self::parse_config(config_path).await?;
        Ok(Self { path, config })
    }

    async fn parse_config(path: Option<PathBuf>) -> anyhow::Result<(PathBuf, Config)> {
        let path = path.unwrap_or_else(|| {
            xdg::BaseDirectories::with_prefix("bo")
                .unwrap()
                .place_config_file("config.toml")
                .unwrap()
        });
        let config = toml::from_str::<Config>(&tokio::fs::read_to_string(&path).await?)?;

        Ok((path, config))
    }

    /// Open the URL matching the given name.
    pub fn open(&self, name: &str) -> anyhow::Result<()> {
        let url_config = self
            .get_url_config(name)
            .ok_or_else(|| anyhow::anyhow!("Bookmark not found: {name}"))?;

        let browser = match &url_config.browser {
            Some(browser) => browser,
            None => self.default_browser.as_str(),
        };

        open::with(&url_config.url, browser).map_err(Into::into)
    }

    /// Open the URL matching the given name, with the rest of the arguments as the query.
    pub fn search(&self, args: &[String]) -> anyhow::Result<()> {
        let first = args.first().unwrap(); // Safe to unwrap because `args` is not empty. Should be checked in the caller.
        let rest = args.iter().skip(1).cloned().collect::<Vec<_>>(); // rest as an argument to the URL

        let url_config = self
            .get_url_config(first)
            .ok_or_else(|| anyhow::anyhow!("Bookmark not found: {first}"))?;

        let browser = match &url_config.browser {
            Some(browser) => browser,
            None => self.default_browser.as_str(),
        };

        if url_config.url.contains("{query}") {
            let url = url_config.url.replace("{query}", &rest.join(" "));
            open::with(url, browser).map_err(Into::into)
        } else {
            self.open(first)
        }
    }

    /// Open an interactive prompt to select a bookmark to open.
    pub fn open_prompt(&self) -> anyhow::Result<()> {
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
        let name = regex::Regex::new(r":\shttp.+\s\(in\s(\w+)\)$")
            .unwrap()
            .replace(name, "");
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
