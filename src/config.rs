use config::{Config, ConfigError, Environment};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub log: LoggingConfig,
    #[serde(default)]
    pub token: TokenConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub app: AppInfo,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    pub dir: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: u64,
    pub acquire_timeout: u64,
    pub idle_timeout: u64,
    pub max_lifetime: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenConfig {
    pub secret_key: String,
    pub expiration: String,
    pub refresh_secret_key: String,
    pub refresh_expiration: String,
    pub issuer: String,
    pub audience: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub environment: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            dir: "logs".to_string(),
        }
    }
}

impl Default for TokenConfig {
    fn default() -> Self {
        Self {
            secret_key: "secret".to_string(),
            expiration: "15m".to_string(),
            refresh_secret_key: "refresh_secret".to_string(),
            refresh_expiration: "7d".to_string(),
            issuer: "rust".to_string(),
            audience: "vue".to_string(),
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgres://postgres@localhost:5432/rust_starterkit".to_string(),
            max_connections: 15,
            min_connections: 2,
            connect_timeout: 10,
            acquire_timeout: 8,
            idle_timeout: 300,
            max_lifetime: 1800,
        }
    }
}

impl Default for AppInfo {
    fn default() -> Self {
        Self {
            name: "rust-starterkit".to_string(),
            version: "0.1.0".to_string(),
            environment: "development".to_string(),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            log: LoggingConfig::default(),
            token: TokenConfig::default(),
            database: DatabaseConfig::default(),
            app: AppInfo::default(),
        }
    }
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let env_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".env");
        let _ = dotenvy::from_path(&env_path).ok();
        let _ = dotenvy::dotenv().ok();

        Config::builder()
            .add_source(Config::try_from(&AppConfig::default())?)
            .add_source(
                Environment::with_prefix("APP")
                    .prefix_separator("_")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()?
            .try_deserialize()
    }
}
