use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub data_dir: PathBuf,
    pub host: String,
    pub port: u16,
    pub log_level: String,
}

impl Config {
    pub fn from_env() -> Self {
        let data_dir = std::env::var("DIVINELIGHT_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::data_local_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("divinelight")
            });

        Self {
            data_dir,
            host: std::env::var("DIVINELIGHT_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: std::env::var("DIVINELIGHT_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
            log_level: std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "divinelight=info,tower_http=info".to_string()),
        }
    }

    pub fn socket_addr(&self) -> std::net::SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .expect("Invalid address")
    }
}
