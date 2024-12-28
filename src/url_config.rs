use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct UrlConfig {
    pub url: String,
    pub browser: Option<String>,
}
