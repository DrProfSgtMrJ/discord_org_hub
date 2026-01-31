use crate::components::{DownloadButton, Leaderboard};
use crate::services::DiscordService;
use dioxus::prelude::*;
use dioxus_router::hooks::use_navigator;

#[derive(Clone, PartialEq)]
struct UserInfo {
    id: String,
    username: String,
    avatar_url: Option<String>,
}

/// The Home page component that will be rendered when the current route is `[Route::Home]`
#[component]
pub fn Home() -> Element {
    let nav = use_navigator();

    // Check URL parameters for OAuth callback status
    let mut show_leaderboard = use_signal(|| false);
    let mut auth_status = use_signal(|| String::new());
    let mut is_logged_in = use_signal(|| false);
    let mut user_info = use_signal(|| None::<UserInfo>);

    // Check for authentication state on page load
    use_effect(move || {
        // Check for auth success in URL query parameters (auth=success&user_id=...)
        if let Some(window) = web_sys::window() {
            if let Ok(href) = window.location().href() {
                if let Ok(url) = web_sys::Url::new(&href) {
                    let search_params = url.search_params();

                    // Check for successful auth redirect
                    if let Some(auth) = search_params.get("auth") {
                        if auth == "success" {
                            if let Some(user_id) = search_params.get("user_id") {
                                auth_status.set("success".to_string());
                                is_logged_in.set(true);
                                show_leaderboard.set(true);

                                // Create user info
                                let username = "Discord User".to_string();
                                let avatar_url = None;

                                user_info.set(Some(UserInfo {
                                    id: user_id.clone(),
                                    username: username.clone(),
                                    avatar_url: avatar_url.clone(),
                                }));

                                // Store in localStorage
                                let _ = DiscordService::store_auth_state(
                                    &user_id, &username, avatar_url,
                                );
                                return;
                            }
                        }
                    }

                    // Handle OAuth error
                    if let Some(_error) = search_params.get("error") {
                        auth_status.set("error".to_string());
                        return;
                    }
                }
            }
        }

        // No OAuth callback - check localStorage
        if let Some((user_id, username, avatar_url)) = DiscordService::get_stored_auth_state() {
            is_logged_in.set(true);
            user_info.set(Some(UserInfo {
                id: user_id,
                username,
                avatar_url,
            }));
            show_leaderboard.set(true);
        }
    });

    let handle_discord_login = move |_| {
        let _ = DiscordService::start_oauth_flow();
    };

    let handle_logout = move |_| {
        is_logged_in.set(false);
        user_info.set(None);
        auth_status.set(String::new());
        show_leaderboard.set(false);

        // Clear stored authentication state
        let _ = DiscordService::clear_stored_auth_state();

        // Clear URL parameters
        let _ = DiscordService::clear_url_params();
    };

    rsx! {
        div { class: "home-container",
            // Header with auth controls
            header { class: "header",
                div { class: "header-content",
                    h1 { class: "title", "Discord Organization Hub" }

                    div { class: "auth-controls",
                        if is_logged_in() {
                            div { class: "user-info",
                                if let Some(user) = user_info() {
                                    div { class: "user-avatar-name",
                                        if let Some(avatar) = &user.avatar_url {
                                            img {
                                                src: avatar.clone(),
                                                alt: "User Avatar",
                                                class: "user-avatar"
                                            }
                                        }
                                        span { class: "username", "Welcome, {user.username}!" }
                                    }
                                }
                                button {
                                    class: "logout-btn",
                                    onclick: handle_logout,
                                    "Logout"
                                }
                            }
                        } else {
                            div { class: "auth-buttons",
                                button {
                                    class: "discord-login-btn",
                                    onclick: handle_discord_login,
                                    "🎮 Sign in with Discord"
                                }



                                p { class: "auth-description",
                                    "Connect your Discord account to access the organization hub"
                                }

                            }
                        }
                    }
                }
            }

            // Auth status messages
            if auth_status() == "success" {
                div { class: "auth-success",
                    "✅ Successfully signed in! Welcome to the Discord Organization Hub."
                }
            }
            if auth_status() == "error" {
                div { class: "auth-error",
                    "❌ Sign in failed. Please try again."
                }
            }

            // Show simple error message
            if let Some(window) = web_sys::window() {
                if let Ok(href) = window.location().href() {
                    if href.contains("error=") {
                        div { class: "auth-error", style: "margin-top: 10px;",
                            "❌ Authentication failed. Please try again."
                            button {
                                style: "margin-left: 10px; padding: 5px 10px; background: #666; color: white; border: none; border-radius: 4px; cursor: pointer;",
                                onclick: move |_| {
                                    if let Some(window) = web_sys::window() {
                                        let _ = window.location().set_href("/");
                                    }
                                },
                                "Try Again"
                            }
                        }
                    }
                }
            }

            // Main content - show different content based on auth status
            div { class: "main-content",
                if is_logged_in() {
                    // Authenticated user content
                    div { class: "dashboard",
                        div { class: "welcome-back-section",
                            h2 { "Welcome back!" }
                            p { "Your Discord account is connected. Download the bot to get started with your organization hub." }
                        }

                        DownloadButton {}

                        div { class: "dashboard-grid",
                            Leaderboard {}

                            div { class: "quick-actions",
                                h3 { "Quick Actions" }
                                div { class: "action-buttons",
                                    button {
                                        class: "view-leaderboard-btn",
                                        onclick: move |_| { nav.push("/leaderboard"); },
                                        "📊 View Full Leaderboard"
                                    }
                                    button {
                                        class: "action-btn",
                                        "⚙️ Bot Settings"
                                    }
                                    button {
                                        class: "action-btn",
                                        "👥 Manage Members"
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // Unauthenticated user content
                    div { class: "welcome-content",
                        div { class: "hero-section",
                            h2 { "Welcome to Discord Organization Hub" }
                            p {
                                "Manage your Discord community, track member engagement, "
                                "and access exclusive features by connecting your Discord account."
                            }

                            div { class: "features-preview",
                                div { class: "feature-card",
                                    h3 { "📊 Leaderboards" }
                                    p { "Track member activity and engagement" }
                                }
                                div { class: "feature-card",
                                    h3 { "🤖 Bot Integration" }
                                    p { "Seamless Discord bot management" }
                                }
                                div { class: "feature-card",
                                    h3 { "📈 Analytics" }
                                    p { "Detailed server insights and statistics" }
                                }
                            }
                        }

                        div { class: "cta-section",
                            button {
                                class: "discord-login-btn large",
                                onclick: handle_discord_login,
                                "🎮 Get Started with Discord"
                            }
                        }
                    }
                }
            }
        }
    }
}
