use std::collections::BTreeMap;

use crate::UrlConfig;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
    /// Default browser to use.
    pub default_browser: String,

    /// Bookmark aliases
    pub aliases: Option<BTreeMap<String, String>>,

    /// Bookmark name to URL mappings.
    pub bookmarks: BTreeMap<String, UrlConfig>,
}
