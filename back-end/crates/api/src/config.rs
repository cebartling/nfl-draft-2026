use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub seed_api_key: Option<String>,
    /// Comma-separated list of allowed CORS origins.
    /// If empty or unset, defaults to common development origins.
    pub cors_origins: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8000
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let host = std::env::var("API_HOST").unwrap_or_else(|_| default_host());
        let port = std::env::var("API_PORT")
            .unwrap_or_else(|_| default_port().to_string())
            .parse()
            .expect("API_PORT must be a valid number");

        let seed_api_key = std::env::var("SEED_API_KEY").ok().filter(|s| !s.is_empty());

        let cors_origins = std::env::var("CORS_ORIGINS")
            .ok()
            .filter(|s| !s.is_empty())
            .map(|s| s.split(',').map(|o| o.trim().to_string()).collect())
            .unwrap_or_else(|| {
                vec![
                    "http://localhost:5173".to_string(),
                    "http://localhost:3000".to_string(),
                    "http://localhost:8080".to_string(),
                ]
            });

        Ok(Config {
            server: ServerConfig { host, port },
            database: DatabaseConfig { url: database_url },
            seed_api_key,
            cors_origins,
        })
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        assert_eq!(default_host(), "0.0.0.0");
        assert_eq!(default_port(), 8000);
    }

    #[test]
    fn test_server_address() {
        let config = Config {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 3000,
            },
            database: DatabaseConfig {
                url: "postgresql://localhost/test".to_string(),
            },
            seed_api_key: None,
            cors_origins: vec!["http://localhost:5173".to_string()],
        };

        assert_eq!(config.server_address(), "127.0.0.1:3000");
    }
}
