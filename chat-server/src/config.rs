use std::fs::File;

use anyhow::{bail, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub db: DbConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct DbConfig {
    pub dsn: String,
}

impl AppConfig {
    pub fn try_load() -> Result<Self> {
        // read from ./config.yaml or /etc/config/easy-chat.yaml or env EASY_CHAT_CONFIG
        let ret = match (
            File::open("./config.yaml"),
            File::open("/etc/config/easy-chat.yaml"),
            std::env::var("EASY_CHAT_CONFIG"),
        ) {
            (Ok(file), _, _) => serde_yaml::from_reader(&file)?,
            (_, Ok(file), _) => serde_yaml::from_reader(&file)?,
            (_, _, Ok(path)) => serde_yaml::from_reader(&File::open(path)?)?,
            _ => bail!("Failed to open config file"),
        };

        Ok(ret)
    }
}
