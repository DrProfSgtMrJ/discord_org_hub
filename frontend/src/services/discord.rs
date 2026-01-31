use crate::config::Config;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use web_sys::{window, Url};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub discriminator: String,
    pub global_name: Option<String>,
    pub avatar: Option<String>,
    pub email: Option<String>,
    pub verified: Option<bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuthCallbackParams {
    pub code: Option<String>,
    pub error: Option<String>,
}

pub struct DiscordService;

impl DiscordService {
    /// Generate Discord OAuth URL and redirect user to Discord
    pub fn start_oauth_flow() -> Result<(), JsValue> {
        let oauth_url = Config::discord_oauth_url();

        // Log the exact OAuth URL we're using
        web_sys::console::log_1(&"🔗 OAUTH URL BEING USED:".into());
        web_sys::console::log_1(&oauth_url.clone().into());
        web_sys::console::log_1(&format!("✓ Client ID: {}", Config::discord_client_id()).into());
        web_sys::console::log_1(
            &format!("✓ Redirect URI: {}", Config::discord_redirect_uri()).into(),
        );
        web_sys::console::log_1(
            &format!(
                "✓ Encoded Redirect URI: {}",
                crate::config::urlencoding::encode(Config::discord_redirect_uri())
            )
            .into(),
        );

        if let Some(window) = window() {
            window.location().set_href(&oauth_url)?;
        }

        Ok(())
    }

    /// Parse URL parameters from OAuth callback (handles both token and code flows)
    pub fn parse_callback_params() -> AuthCallbackParams {
        let mut params = AuthCallbackParams {
            code: None,
            error: None,
        };

        if let Some(window) = window() {
            let location = window.location();
            if let Ok(href) = location.href() {
                web_sys::console::log_1(&format!("🔍 Parsing callback URL: {}", href).into());

                // Check query parameters for authorization code (code flow)
                if let Ok(url) = Url::new(&href) {
                    let search_params = url.search_params();

                    // Log parameters we're specifically looking for
                    web_sys::console::log_1(&"🔍 Checking for specific parameters:".into());

                    // Check for OAuth code (authorization code flow)
                    if let Some(code) = search_params.get("code") {
                        web_sys::console::log_1(
                            &format!("✅ Found authorization code: {}", code).into(),
                        );
                        params.code = Some(code);
                    } else {
                        web_sys::console::log_1(&"❌ No 'code' parameter found".into());
                    }

                    // Check for OAuth error
                    if let Some(error) = search_params.get("error") {
                        web_sys::console::log_1(&format!("❌ Found OAuth error: {}", error).into());
                        params.error = Some(error);
                    } else {
                        web_sys::console::log_1(&"✅ No error parameter found".into());
                    }
                } else {
                    web_sys::console::log_1(&"❌ Failed to parse URL!".into());
                }
            }
        }

        params
    }

    /// Check if current URL indicates successful authentication
    pub fn is_auth_success() -> bool {
        if let Some(window) = window() {
            let location = window.location();
            if let Ok(href) = location.href() {
                if let Ok(url) = Url::new(&href) {
                    let search_params = url.search_params();
                    if let Some(auth_param) = search_params.get("auth") {
                        return auth_param == "success";
                    }
                }
            }
        }
        false
    }

    /// Check if current URL indicates authentication error
    pub fn is_auth_error() -> bool {
        if let Some(window) = window() {
            let location = window.location();
            if let Ok(href) = location.href() {
                if let Ok(url) = Url::new(&href) {
                    let search_params = url.search_params();
                    return search_params.get("error").is_some();
                }
            }
        }
        false
    }

    /// Clear URL parameters by replacing current state
    pub fn clear_url_params() -> Result<(), JsValue> {
        if let Some(window) = window() {
            if let Ok(history) = window.history() {
                history.replace_state_with_url(&JsValue::NULL, "", Some("/"))?;
            }
        }
        Ok(())
    }

    /// Generate avatar URL for Discord user
    pub fn get_avatar_url(user_id: &str, avatar_hash: &str) -> String {
        format!(
            "https://cdn.discordapp.com/avatars/{}/{}.png",
            user_id, avatar_hash
        )
    }

    /// Get default Discord avatar URL
    pub fn get_default_avatar_url(discriminator: &str) -> String {
        let avatar_id = discriminator.parse::<u32>().unwrap_or(0) % 5;
        format!("https://cdn.discordapp.com/embed/avatars/{}.png", avatar_id)
    }

    /// Store authentication state in localStorage
    pub fn store_auth_state(
        user_id: &str,
        username: &str,
        avatar_url: Option<String>,
    ) -> Result<(), JsValue> {
        if let Some(window) = window() {
            if let Ok(Some(storage)) = window.local_storage() {
                storage.set_item("discord_user_id", user_id)?;
                storage.set_item("discord_username", username)?;

                if let Some(avatar) = avatar_url {
                    storage.set_item("discord_avatar_url", &avatar)?;
                } else {
                    storage.remove_item("discord_avatar_url")?;
                }

                storage.set_item("is_logged_in", "true")?;
            }
        }
        Ok(())
    }

    /// Retrieve authentication state from localStorage
    pub fn get_stored_auth_state() -> Option<(String, String, Option<String>)> {
        if let Some(window) = window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let is_logged_in = storage.get_item("is_logged_in").ok().flatten();

                if is_logged_in.as_deref() == Some("true") {
                    let user_id = storage.get_item("discord_user_id").ok().flatten();
                    let username = storage.get_item("discord_username").ok().flatten();
                    let avatar_url = storage.get_item("discord_avatar_url").ok().flatten();

                    if let (Some(user_id), Some(username)) = (user_id, username) {
                        return Some((user_id, username, avatar_url));
                    }
                }
            }
        }
        None
    }

    /// Clear stored authentication state
    pub fn clear_stored_auth_state() -> Result<(), JsValue> {
        if let Some(window) = window() {
            if let Ok(Some(storage)) = window.local_storage() {
                storage.remove_item("discord_user_id")?;
                storage.remove_item("discord_username")?;
                storage.remove_item("discord_avatar_url")?;
                storage.remove_item("is_logged_in")?;
            }
        }
        Ok(())
    }
}

// URL encoding is now handled by the config module
