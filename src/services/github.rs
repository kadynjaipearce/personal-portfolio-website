use crate::error::{AppError, AppResult};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::RwLock;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubStats {
    pub public_repos: i32,
    pub followers: i32,
    pub following: i32,
    pub total_stars: i32,
    pub avatar_url: String,
    pub bio: Option<String>,
    pub recent_repos: Vec<RepoInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoInfo {
    pub name: String,
    pub description: Option<String>,
    pub stars: i32,
    pub forks: i32,
    pub language: Option<String>,
    pub url: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
struct GitHubUser {
    pub public_repos: i32,
    pub followers: i32,
    pub following: i32,
    pub avatar_url: String,
    pub bio: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubRepo {
    pub name: String,
    pub description: Option<String>,
    pub stargazers_count: i32,
    pub forks_count: i32,
    pub language: Option<String>,
    pub html_url: String,
    pub updated_at: String,
    pub fork: bool,
}

const CACHE_TTL: Duration = Duration::from_secs(24 * 3600);

static CACHE: Lazy<RwLock<Option<(String, GitHubStats, Instant)>>> =
    Lazy::new(|| RwLock::new(None));

pub struct GitHubService;

impl GitHubService {
    /// Returns cached stats if present and less than 24h old, otherwise fetches and caches.
    pub async fn get_user_stats(username: &str) -> AppResult<GitHubStats> {
        let now = Instant::now();
        {
            let guard = CACHE.read().map_err(|_| AppError::InternalError("cache lock".into()))?;
            if let Some((cached_username, ref stats, expires_at)) = guard.as_ref() {
                if cached_username == username && now < *expires_at {
                    return Ok(stats.clone());
                }
            }
        }
        let stats = Self::fetch_user_stats(username).await?;
        {
            let mut guard = CACHE.write().map_err(|_| AppError::InternalError("cache lock".into()))?;
            *guard = Some((username.to_string(), stats.clone(), now + CACHE_TTL));
        }
        Ok(stats)
    }

    async fn fetch_user_stats(username: &str) -> AppResult<GitHubStats> {
        let client = reqwest::Client::new();

        // Fetch user info
        let user: GitHubUser = client
            .get(format!("https://api.github.com/users/{}", username))
            .header("User-Agent", "portfolio-app")
            .send()
            .await?
            .json()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to fetch GitHub user: {}", e)))?;

        // Fetch repos
        let repos: Vec<GitHubRepo> = client
            .get(format!(
                "https://api.github.com/users/{}/repos?sort=updated&per_page=10",
                username
            ))
            .header("User-Agent", "portfolio-app")
            .send()
            .await?
            .json()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to fetch GitHub repos: {}", e)))?;

        // Calculate total stars
        let total_stars: i32 = repos.iter().map(|r| r.stargazers_count).sum();

        // Filter and map repos (exclude forks)
        let recent_repos: Vec<RepoInfo> = repos
            .into_iter()
            .filter(|r| !r.fork)
            .take(5)
            .map(|r| RepoInfo {
                name: r.name,
                description: r.description,
                stars: r.stargazers_count,
                forks: r.forks_count,
                language: r.language,
                url: r.html_url,
                updated_at: r.updated_at,
            })
            .collect();

        Ok(GitHubStats {
            public_repos: user.public_repos,
            followers: user.followers,
            following: user.following,
            total_stars,
            avatar_url: user.avatar_url,
            bio: user.bio,
            recent_repos,
        })
    }
}
