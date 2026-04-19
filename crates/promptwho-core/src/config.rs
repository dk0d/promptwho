use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;

use figment::{
    Figment,
    providers::{Env, Format, Serialized, Toml},
};
use serde::{Deserialize, Serialize};

const CONFIG_FILE_NAME: &str = "promptwho.toml";

fn derive_xdg_config_file() -> Option<PathBuf> {
    if let Some(xdg_config_home) = std::env::var_os("XDG_CONFIG_HOME") {
        let mut path = PathBuf::from(xdg_config_home);
        path.push("promptwho");
        path.push(CONFIG_FILE_NAME);
        return Some(path);
    }

    if let Some(home_dir) = dirs::home_dir() {
        let mut path = home_dir;
        path.push(".config");
        path.push("promptwho");
        path.push(CONFIG_FILE_NAME);
        return Some(path);
    }

    None
}

fn default_server_host() -> IpAddr {
    IpAddr::V4(Ipv4Addr::LOCALHOST)
}

fn default_server_port() -> u16 {
    8765
}

fn default_surreal_endpoint() -> String {
    "surrealkv://.promptwho/db".to_string()
}

fn default_namespace() -> String {
    "promptwho".to_string()
}

fn default_database() -> String {
    "promptwho".to_string()
}

fn default_false() -> bool {
    false
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PromptwhoConfig {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_server_host")]
    pub host: IpAddr,

    #[serde(default = "default_server_port")]
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_server_host(),
            port: default_server_port(),
        }
    }
}

impl ServerConfig {
    pub fn listen_addr(&self) -> SocketAddr {
        SocketAddr::new(self.host, self.port)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "backend", rename_all = "snake_case")]
pub enum StorageConfig {
    Surreal {
        #[serde(flatten)]
        config: SurrealConfig,
    },
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self::Surreal {
            config: SurrealConfig::default(),
        }
    }
}

impl StorageConfig {
    pub fn surreal(&self) -> Option<&SurrealConfig> {
        match self {
            Self::Surreal { config } => Some(config),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurrealConfig {
    #[serde(default = "default_surreal_endpoint")]
    pub endpoint: String,

    #[serde(default = "default_namespace")]
    pub namespace: String,

    #[serde(default = "default_database")]
    pub database: String,

    #[serde(default)]
    pub username: Option<String>,

    #[serde(default)]
    pub password: Option<String>,

    #[serde(default = "default_false")]
    pub vector_enabled: bool,

    #[serde(default = "default_false")]
    pub sync_enabled: bool,
}

impl Default for SurrealConfig {
    fn default() -> Self {
        Self {
            endpoint: default_surreal_endpoint(),
            namespace: default_namespace(),
            database: default_database(),
            username: None,
            password: None,
            vector_enabled: false,
            sync_enabled: false,
        }
    }
}

impl PromptwhoConfig {
    pub fn load(config_path: Option<String>) -> Self {
        let config_path = config_path
            .map(PathBuf::from)
            .or_else(derive_xdg_config_file);

        let mut figment = Figment::from(Serialized::defaults(Self::default()));

        if let Some(path) = config_path {
            figment = figment.merge(Toml::file(path));
        } else {
            figment = figment.merge(Toml::file(CONFIG_FILE_NAME));
        }

        figment = figment.merge(Env::prefixed("PROMPTWHO_"));

        figment.extract().expect("Failed to load configuration")
    }
}
