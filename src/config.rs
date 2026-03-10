use once_cell::sync::Lazy;
use std::env;

pub static CONFIG: Lazy<Config> = Lazy::new(Config::from_env);

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub database_ns: String,
    pub database_db: String,
    pub github_client_id: String,
    pub github_client_secret: String,
    pub github_redirect_uri: String,
    pub admin_github_username: String,
    pub session_secret: String,
    pub contact_email: String,
    pub resend_api_key: String,
    pub resend_from_email: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .expect("PORT must be a number"),
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "file:///app/data/portfolio.db".to_string()),
            database_ns: env::var("DATABASE_NS").unwrap_or_else(|_| "portfolio".to_string()),
            database_db: env::var("DATABASE_DB").unwrap_or_else(|_| "main".to_string()),
            github_client_id: env::var("GITHUB_CLIENT_ID").unwrap_or_default(),
            github_client_secret: env::var("GITHUB_CLIENT_SECRET").unwrap_or_default(),
            github_redirect_uri: env::var("GITHUB_REDIRECT_URI")
                .unwrap_or_else(|_| "http://localhost:8080/auth/github/callback".to_string()),
            admin_github_username: env::var("ADMIN_GITHUB_USERNAME")
                .unwrap_or_else(|_| "kadynjaipearce".to_string()),
            session_secret: env::var("SESSION_SECRET").unwrap_or_else(|_| {
                "development_secret_key_change_in_production_please".to_string()
            }),
            contact_email: env::var("CONTACT_EMAIL")
                .unwrap_or_else(|_| "kadynjaipearce@gmail.com".to_string()),
            resend_api_key: env::var("RESEND_API_KEY").unwrap_or_default(),
            resend_from_email: env::var("RESEND_FROM_EMAIL")
                .unwrap_or_else(|_| "noreply@kadynpearce.dev".to_string()),
        }
    }

    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
