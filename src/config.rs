use once_cell::sync::Lazy;
use std::env;

pub static CONFIG: Lazy<Config> = Lazy::new(Config::from_env);

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub github_client_id: String,
    pub github_client_secret: String,
    pub github_redirect_uri: String,
    pub admin_github_username: String,
    pub session_secret: String,
    pub contact_email: String,
    pub resend_api_key: String,
    pub resend_from_email: String,
    pub site_base_url: String,
    pub twitter_access_token: Option<String>,
}

fn build_database_url() -> String {
    let inject_password = |url: &str, password: &str| -> String {
        let after_scheme = url.find("://").map(|i| i + 3).unwrap_or(0);
        if let Some(at) = url[after_scheme..].find('@').map(|i| after_scheme + i) {
            let user_part = &url[after_scheme..at];
            if !user_part.contains(':') {
                let mut built = url.to_string();
                built.insert_str(at, &format!(":{}", password));
                return built;
            }
        }
        url.to_string()
    };

    let password_var = || {
        env::var("SUPABASE_SERVICE_KEY")
            .or_else(|_| env::var("DATABASE_SERVICE_KEY"))
            .or_else(|_| env::var("DATABASE_PASSWORD"))
    };

    if let Ok(url) = env::var("DATABASE_URL") {
        if let Ok(password) = password_var() {
            if !password.is_empty() {
                return inject_password(&url, &password);
            }
        }
        return url;
    }
    if let (Ok(url), Ok(key)) = (env::var("SUPABASE_DB_URL"), password_var()) {
        if !key.is_empty() {
            return inject_password(&url, &key);
        }
        return url;
    }
    "postgres://localhost/portfolio".to_string()
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .expect("PORT must be a number"),
            database_url: build_database_url(),
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
            site_base_url: env::var("SITE_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            twitter_access_token: env::var("TWITTER_ACCESS_TOKEN").ok().filter(|s| !s.is_empty()),
        }
    }

    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
