//! Configuration module for backend environment variables
//!
//! This module centralizes all environment variable access for the backend,
//! making it easy to manage URLs, secrets, and configuration in one place.

use std::env;

/// Backend configuration struct containing all environment-based settings
pub struct Config;

impl Config {
    // Server configuration
    /// Server host address
    pub fn host() -> String {
        env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string())
    }

    /// Server port
    pub fn port() -> String {
        env::var("PORT").unwrap_or_else(|_| "8080".to_string())
    }

    /// Server base URL
    pub fn server_url() -> String {
        format!("http://{}:{}", Self::host(), Self::port())
    }

    // Database configuration
    /// Database connection URL
    pub fn database_url() -> String {
        env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://discord_user:password@localhost:5432/discord_org_hub".to_string()
        })
    }

    // JWT configuration
    /// JWT secret for authentication
    pub fn jwt_secret() -> String {
        env::var("JWT_SECRET").unwrap_or_else(|_| "your_super_secret_jwt_key_here".to_string())
    }

    // Discord OAuth configuration
    /// Discord OAuth client ID
    pub fn discord_client_id() -> String {
        env::var("DISCORD_CLIENT_ID").expect("DISCORD_CLIENT_ID must be set in environment")
    }

    /// Discord OAuth client secret
    pub fn discord_client_secret() -> String {
        env::var("DISCORD_CLIENT_SECRET").expect("DISCORD_CLIENT_SECRET must be set in environment")
    }

    /// Discord OAuth redirect URI
    pub fn discord_redirect_uri() -> String {
        env::var("DISCORD_REDIRECT_URI")
            .unwrap_or_else(|_| "http://localhost:8081/auth/discord/callback".to_string())
    }

    // Frontend URLs
    /// Frontend base URL
    pub fn frontend_url() -> String {
        env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:8081".to_string())
    }

    /// Frontend callback URL
    pub fn frontend_callback_url() -> String {
        env::var("FRONTEND_CALLBACK_URL")
            .unwrap_or_else(|_| "http://localhost:8081/auth/discord/callback".to_string())
    }

    // Backend URLs
    /// Backend base URL
    pub fn backend_url() -> String {
        env::var("BACKEND_URL").unwrap_or_else(|_| Self::server_url())
    }

    /// Backend API URL
    pub fn backend_api_url() -> String {
        env::var("BACKEND_API_URL").unwrap_or_else(|_| format!("{}/api", Self::backend_url()))
    }

    /// Backend auth URL
    pub fn backend_auth_url() -> String {
        env::var("BACKEND_AUTH_URL").unwrap_or_else(|_| format!("{}/auth", Self::backend_url()))
    }

    // Discord API URLs
    /// Discord OAuth token exchange URL
    pub fn discord_token_url() -> String {
        env::var("DISCORD_TOKEN_URL")
            .unwrap_or_else(|_| "https://discord.com/api/oauth2/token".to_string())
    }

    /// Discord user info API URL
    pub fn discord_user_api_url() -> String {
        env::var("DISCORD_USER_API_URL")
            .unwrap_or_else(|_| "https://discord.com/api/users/@me".to_string())
    }

    /// Discord API base URL
    pub fn discord_api_base_url() -> String {
        env::var("DISCORD_API_BASE_URL").unwrap_or_else(|_| "https://discord.com/api".to_string())
    }

    // CORS configuration
    /// Allowed CORS origins
    pub fn allowed_origins() -> Vec<String> {
        env::var("ALLOWED_ORIGINS")
            .unwrap_or_else(|_| {
                "http://localhost:5173,http://localhost:3000,http://localhost:8081".to_string()
            })
            .split(',')
            .map(|s| s.trim().to_string())
            .collect()
    }

    // Environment detection
    /// Current environment (development, staging, production)
    pub fn environment() -> String {
        env::var("NODE_ENV").unwrap_or_else(|_| "development".to_string())
    }

    /// Check if we're in development mode
    pub fn is_development() -> bool {
        Self::environment() == "development"
    }

    /// Check if we're in production mode
    pub fn is_production() -> bool {
        Self::environment() == "production"
    }

    /// Check if we're in staging mode
    pub fn is_staging() -> bool {
        Self::environment() == "staging"
    }
}

/// Helper functions for building URLs and endpoints
impl Config {
    /// Build a frontend route URL
    pub fn frontend_route(path: &str) -> String {
        format!("{}/{}", Self::frontend_url(), path.trim_start_matches('/'))
    }

    /// Build a backend API endpoint URL
    pub fn backend_api_endpoint(path: &str) -> String {
        format!(
            "{}/{}",
            Self::backend_api_url(),
            path.trim_start_matches('/')
        )
    }

    /// Build a Discord API endpoint URL
    pub fn discord_api_endpoint(path: &str) -> String {
        format!(
            "{}/{}",
            Self::discord_api_base_url(),
            path.trim_start_matches('/')
        )
    }

    /// Get success redirect URL for OAuth
    pub fn oauth_success_redirect(user_id: &str) -> String {
        format!("{}/?auth=success&user_id={}", Self::frontend_url(), user_id)
    }

    /// Get error redirect URL for OAuth
    pub fn oauth_error_redirect(error: &str) -> String {
        format!("{}/?error={}", Self::frontend_url(), error)
    }

    /// Get server bind address
    pub fn bind_address() -> String {
        format!("{}:{}", Self::host(), Self::port())
    }
}

/// Configuration validation
impl Config {
    /// Validate that all required environment variables are set
    pub fn validate() -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Check required environment variables
        if env::var("DISCORD_CLIENT_ID").is_err() {
            errors.push("DISCORD_CLIENT_ID is required".to_string());
        }

        if env::var("DISCORD_CLIENT_SECRET").is_err() {
            errors.push("DISCORD_CLIENT_SECRET is required".to_string());
        }

        if env::var("DATABASE_URL").is_err() && Self::is_production() {
            errors.push("DATABASE_URL is required in production".to_string());
        }

        if env::var("JWT_SECRET").is_err() && Self::is_production() {
            errors.push("JWT_SECRET should be set in production".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Print configuration summary (excluding secrets)
    pub fn print_summary() {
        println!("🔧 Backend Configuration:");
        println!("   Environment: {}", Self::environment());
        println!("   Server: {}", Self::server_url());
        println!(
            "   Database: {}",
            Self::database_url().split('@').next().unwrap_or("***")
        );
        println!("   Frontend: {}", Self::frontend_url());
        println!("   Discord Client ID: {}", Self::discord_client_id());
        println!("   Discord Redirect: {}", Self::discord_redirect_uri());

        if Self::is_development() {
            println!("   CORS Origins: {:?}", Self::allowed_origins());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_default_values() {
        // Test that default values are returned when env vars are not set
        unsafe {
            env::remove_var("HOST");
            env::remove_var("PORT");
        }

        assert_eq!(Config::host(), "127.0.0.1");
        assert_eq!(Config::port(), "8080");
    }

    #[test]
    fn test_url_builders() {
        unsafe {
            env::set_var("FRONTEND_URL", "https://example.com");
        }
        assert_eq!(
            Config::frontend_route("dashboard"),
            "https://example.com/dashboard"
        );

        unsafe {
            env::set_var("BACKEND_URL", "https://api.example.com");
        }
        assert_eq!(
            Config::backend_api_endpoint("users"),
            "https://api.example.com/api/users"
        );
    }

    #[test]
    fn test_oauth_redirects() {
        unsafe {
            env::set_var("FRONTEND_URL", "https://app.example.com");
        }

        assert_eq!(
            Config::oauth_success_redirect("user123"),
            "https://app.example.com/?auth=success&user_id=user123"
        );

        assert_eq!(
            Config::oauth_error_redirect("invalid_request"),
            "https://app.example.com/?error=invalid_request"
        );
    }
}
