use crate::config::CONFIG;
use tracing::warn;

const TWEET_MAX_LEN: usize = 280;

pub struct TwitterService;

impl TwitterService {
    /// Post a tweet when a blog post is published. Skips if Twitter is not configured.
    pub async fn post_new_blog_post(title: &str, slug: &str) -> Result<(), String> {
        let token = match &CONFIG.twitter_access_token {
            Some(t) if !t.is_empty() => t.as_str(),
            _ => return Ok(()),
        };

        let base = CONFIG.site_base_url.trim_end_matches('/');
        let url = format!("{}/blog/{}", base, slug);
        let sep = "\n\n";
        let max_title = TWEET_MAX_LEN.saturating_sub(sep.len()).saturating_sub(url.len());
        let text = if title.len() <= max_title {
            format!("{}{}{}", title, sep, url)
        } else {
            let truncated: String = title.chars().take(max_title.saturating_sub(1)).collect();
            format!("{}…{}{}", truncated.trim_end(), sep, url)
        };

        let client = reqwest::Client::new();
        let res = client
            .post("https://api.twitter.com/2/tweets")
            .bearer_auth(token)
            .json(&serde_json::json!({ "text": text }))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            warn!("Twitter API error {}: {}", status, body);
            return Err(format!("Twitter API {}: {}", status, body));
        }
        Ok(())
    }
}
