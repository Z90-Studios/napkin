use ::config::ConfigError;
use serde::{Deserialize, Serialize};
use deadpool_postgres::PoolConfig;

#[derive(Debug, Deserialize, Serialize)]
pub struct NapkinConfig {
    pub app_name: String,
    pub server_addr: String,
    pub pg: deadpool_postgres::Config,
}

impl Default for NapkinConfig {
    fn default() -> Self {
        Self {
            app_name: "Project: Napkin".to_string(),
            server_addr: "127.0.0.1".to_string(),
            pg: deadpool_postgres::Config {
                host: Some("127.0.0.1".to_string()),
                port: Some(5438),
                user: Some("postgres".to_string()),
                password: Some("postgres".to_string()),
                dbname: Some("napkin".to_string()),
                pool: Some(PoolConfig {
                    max_size: 16,
                    ..Default::default()
                }),
                ..Default::default()
            }

        }
    }
}

impl TryFrom<::config::ConfigBuilder<::config::builder::DefaultState>> for NapkinConfig {
    type Error = ConfigError;

    fn try_from(cfg: ::config::ConfigBuilder<::config::builder::DefaultState>) -> Result<Self, ConfigError> {
        cfg.try_into()
    }
}

impl From<NapkinConfig> for ::config::ConfigBuilder<::config::builder::DefaultState> {
    fn from(cfg: NapkinConfig) -> Self {
        ::config::ConfigBuilder::try_from(cfg).unwrap()
    }
}

impl NapkinConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let builder = ::config::Config::builder()
            .set_default("default", "1")?;

        builder.try_into()
        // let mut cfg = ::config::Config::default();
        // cfg.merge(::config::Environment::default())?;
        // cfg.try_into()
    }
}
