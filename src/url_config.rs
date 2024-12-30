#[derive(Debug, Clone, serde::Deserialize)]
pub struct UrlConfig {
    pub url: String,
    pub browser: Option<String>,
}
