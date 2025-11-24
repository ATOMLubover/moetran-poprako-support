use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct AppConfig {
    pub server_host: String,
    pub server_port: u16,
}

impl AppConfig {
    pub fn try_from_file(path: Option<&str>) -> anyhow::Result<Self> {
        let path = path.unwrap_or("app_config.json");

        let content = std::fs::read_to_string(path)?;

        let config: AppConfig = serde_json::from_str(&content)?;

        Ok(config)
    }
}
