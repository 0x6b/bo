#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct UrlConfig {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub browser: Option<String>,
}
