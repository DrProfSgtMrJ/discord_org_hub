//! Configuration module for frontend environment variables
//!
//! This module centralizes all environment variable access for the frontend,
//! making it easy to manage URLs and configuration in one place.

/// Frontend configuration struct containing all environment-based settings
pub struct Config;

impl Config {
    /// Discord OAuth client ID
    pub fn discord_client_id() -> &'static str {
        option_env!("VITE_DISCORD_CLIENT_ID").unwrap_or("1466997290819649619")
    }

    /// Discord OAuth redirect URI
    pub fn discord_redirect_uri() -> &'static str {
        option_env!("VITE_DISCORD_REDIRECT_URI")
            .unwrap_or("http://localhost:8081/auth/discord/callback")
    }

    /// Discord OAuth scopes
    pub fn discord_scopes() -> &'static str {
        "identify"
    }

    /// Backend base URL
    pub fn backend_url() -> &'static str {
        option_env!("VITE_BACKEND_URL").unwrap_or("http://localhost:8080")
    }

    /// Backend API URL
    pub fn backend_api_url() -> &'static str {
        option_env!("VITE_BACKEND_API_URL").unwrap_or("http://localhost:8080/api")
    }

    /// Backend auth URL
    pub fn backend_auth_url() -> &'static str {
        option_env!("VITE_BACKEND_AUTH_URL").unwrap_or("http://localhost:8080/auth")
    }

    /// Discord OAuth authorization URL
    pub fn discord_oauth_base_url() -> &'static str {
        "https://discord.com/oauth2/authorize"
    }

    /// Frontend base URL
    pub fn frontend_url() -> &'static str {
        option_env!("VITE_FRONTEND_URL").unwrap_or("http://localhost:8081")
    }

    /// Environment (development, staging, production)
    pub fn node_env() -> &'static str {
        option_env!("VITE_NODE_ENV").unwrap_or("development")
    }

    /// Get the full Discord OAuth authorization URL
    pub fn discord_oauth_url() -> String {
        format!(
            "{}?client_id={}&response_type=code&redirect_uri={}&scope={}",
            Self::discord_oauth_base_url(),
            Self::discord_client_id(),
            urlencoding::encode(Self::discord_redirect_uri()),
            Self::discord_scopes()
        )
    }

    /// Get the backend Discord exchange URL with code
    pub fn backend_discord_exchange_url(code: &str) -> String {
        format!(
            "{}/discord/exchange?code={}",
            Self::backend_auth_url(),
            code
        )
    }

    /// Get a backend API endpoint URL
    pub fn backend_api_endpoint(path: &str) -> String {
        format!(
            "{}/{}",
            Self::backend_api_url(),
            path.trim_start_matches('/')
        )
    }

    /// Get a frontend route URL
    pub fn frontend_route(path: &str) -> String {
        format!("{}/{}", Self::frontend_url(), path.trim_start_matches('/'))
    }

    /// Check if we're in development mode
    pub fn is_development() -> bool {
        Self::node_env() == "development"
    }

    /// Check if we're in production mode
    pub fn is_production() -> bool {
        Self::node_env() == "production"
    }
}

/// URL encoding helper (re-exported for convenience)
pub mod urlencoding {
    pub fn encode(input: &str) -> String {
        input
            .chars()
            .map(|c| match c {
                'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
                ':' => "%3A".to_string(),
                '/' => "%2F".to_string(),
                '?' => "%3F".to_string(),
                '#' => "%23".to_string(),
                '[' => "%5B".to_string(),
                ']' => "%5D".to_string(),
                '@' => "%40".to_string(),
                '!' => "%21".to_string(),
                '$' => "%24".to_string(),
                '&' => "%26".to_string(),
                '\'' => "%27".to_string(),
                '(' => "%28".to_string(),
                ')' => "%29".to_string(),
                '*' => "%2A".to_string(),
                '+' => "%2B".to_string(),
                ',' => "%2C".to_string(),
                ';' => "%3B".to_string(),
                '=' => "%3D".to_string(),
                ' ' => "%20".to_string(),
                _ => {
                    let bytes = c.to_string().into_bytes();
                    bytes.iter().map(|b| format!("%{:02X}", b)).collect()
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discord_oauth_url() {
        let url = Config::discord_oauth_url();
        assert!(url.contains("discord.com/oauth2/authorize"));
        assert!(url.contains("client_id="));
        assert!(url.contains("response_type=code"));
        assert!(url.contains("scope=identify"));
    }

    #[test]
    fn test_backend_endpoints() {
        let exchange_url = Config::backend_discord_exchange_url("test_code");
        assert!(exchange_url.contains("/discord/exchange?code=test_code"));

        let api_url = Config::backend_api_endpoint("users");
        assert!(api_url.contains("/api/users"));
    }

    #[test]
    fn test_url_encoding() {
        assert_eq!(urlencoding::encode("hello world"), "hello%20world");
        assert_eq!(
            urlencoding::encode("test@example.com"),
            "test%40example.com"
        );
    }
}
