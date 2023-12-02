use ::config::ConfigError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct NapkinConfig {
    pub app_name: String,
    pub server_addr: String,
    pub pg: deadpool_postgres::Config,
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