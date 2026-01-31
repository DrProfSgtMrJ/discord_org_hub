use crate::config::Config;
use dioxus::prelude::*;
use dioxus_router::hooks::use_navigator;

/// Discord OAuth callback handler component
#[component]
pub fn DiscordCallback(code: Option<String>, error: Option<String>) -> Element {
    let nav = use_navigator();

    // Log that the component is being rendered
    web_sys::console::log_1(&"🚀 DiscordCallback component is rendering".into());

    // Process the OAuth callback when component loads
    use_effect(move || {
        let nav = nav.clone();
        let code_clone = code.clone();
        let error_clone = error.clone();
        spawn(async move {
            web_sys::console::log_1(&"🔍 Starting OAuth callback processing".into());
            web_sys::console::log_1(
                &format!(
                    "📋 Props - code: {:?}, error: {:?}",
                    code_clone, error_clone
                )
                .into(),
            );

            // Check for OAuth error first
            if let Some(error) = error_clone {
                web_sys::console::log_1(&format!("❌ OAuth error from props: {}", error).into());
                nav.push(format!("/?error=oauth_failed&details={}", error));
                return;
            }

            // Check for authorization code from props
            if let Some(code) = code_clone {
                web_sys::console::log_1(
                    &format!(
                        "✅ Found authorization code from props: {} (length: {})",
                        code,
                        code.len()
                    )
                    .into(),
                );

                // Send code to backend via API request
                match send_code_to_backend(code).await {
                    Ok(user_id) => {
                        web_sys::console::log_1(
                            &format!("Successfully authenticated user: {}", user_id).into(),
                        );
                        nav.push(format!("/?auth=success&user_id={}", user_id));
                    }
                    Err(e) => {
                        web_sys::console::log_1(&format!("Authentication failed: {}", e).into());
                        nav.push("/?error=token_exchange_failed".to_string());
                    }
                }
            } else {
                web_sys::console::log_1(&"❌ No authorization code found in props".into());
                nav.push("/?error=missing_code".to_string());
            }

            web_sys::console::log_1(&"🏁 OAuth callback processing complete".into());
        });
    });

    rsx! {
        div { class: "callback-container",
            style: "display: flex; justify-content: center; align-items: center; min-height: 100vh; background: #0f1116; color: white; flex-direction: column;",

            div { class: "loading-spinner",
                style: "width: 50px; height: 50px; border: 3px solid #333; border-top: 3px solid #667eea; border-radius: 50%; animation: spin 1s linear infinite; margin-bottom: 20px;",
            }

            h1 { style: "margin: 0 0 10px 0; font-size: 1.5rem;",
                "Processing Discord Authentication..."
            }

            p { style: "margin: 0; color: #888; font-size: 1rem;",
                "Please wait while we complete your sign-in"
            }
        }
    }
}

async fn send_code_to_backend(code: String) -> Result<String, String> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Request, RequestInit, Response};

    // Send authorization code to backend exchange endpoint
    let url = Config::backend_discord_exchange_url(&code);

    let opts = RequestInit::new();
    opts.set_method("GET");

    let request = Request::new_with_str_and_init(&url, &opts)
        .map_err(|e| format!("Failed to create request: {:?}", e))?;

    let window = web_sys::window().ok_or("No window object")?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Backend request failed: {:?}", e))?;

    let resp: Response = resp_value
        .dyn_into()
        .map_err(|e| format!("Failed to cast response: {:?}", e))?;

    if !resp.ok() {
        return Err(format!("Backend exchange failed: {}", resp.status()));
    }

    let text = JsFuture::from(
        resp.text()
            .map_err(|e| format!("Failed to get response: {:?}", e))?,
    )
    .await
    .map_err(|e| format!("Failed to read response: {:?}", e))?;

    let response_str = text.as_string().unwrap_or_default();

    // Parse backend response
    let json = serde_json::from_str::<serde_json::Value>(&response_str)
        .map_err(|e| format!("Failed to parse backend response: {}", e))?;

    let success = json
        .get("success")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if !success {
        let error = json
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown_error");
        return Err(format!("Backend error: {}", error));
    }

    let user_id = json
        .get("user_id")
        .and_then(|v| v.as_str())
        .ok_or("No user_id in backend response")?;

    Ok(user_id.to_string())
}
