use axum::{
    Json, Router,
    extract::{Query, State},
    response::{IntoResponse, Redirect},
    routing::{get, post},
};
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};

mod config;
mod database;
mod handlers;

use config::Config;
use database::{DatabasePool, create_pool, health_check as db_health_check};

// Shared AppState for all handlers
#[derive(Clone)]
pub struct AppState {
    pub db_pool: std::sync::Arc<DatabasePool>,
}

#[derive(Deserialize)]
struct DiscordCallbackQuery {
    code: Option<String>,
    error: Option<String>,
    state: Option<String>,
}

#[derive(Deserialize)]
struct DiscordExchangeQuery {
    code: String,
    frontend_redirect: Option<String>,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    // Initialize database connection pool
    let db_pool = create_pool().await.expect("Failed to create database pool");
    println!("Database connected! Run 'sqlx migrate run' to apply migrations if needed.");

    let state = AppState {
        db_pool: Arc::new(db_pool),
    };

    // API routes
    let api_routes = Router::new()
        .route(
            "/users",
            get(handlers::list_users).post(handlers::create_user),
        )
        .route(
            "/users/:id",
            get(handlers::get_user)
                .put(handlers::update_user)
                .delete(handlers::delete_user),
        )
        .route(
            "/users/discord/:discord_id",
            get(handlers::get_user_by_discord_id),
        )
        .route("/users/stats", get(handlers::get_user_stats))
        // Discord token routes
        .route(
            "/discord-tokens",
            post(handlers::discord_tokens::upsert_discord_token),
        )
        .route(
            "/discord-tokens/user/:user_id",
            get(handlers::discord_tokens::get_discord_token_by_user)
                .put(handlers::discord_tokens::update_discord_token)
                .delete(handlers::discord_tokens::delete_discord_token_by_user),
        )
        .route(
            "/discord-tokens/verify/:user_id",
            get(handlers::discord_tokens::verify_discord_token),
        )
        .route(
            "/discord-tokens/cleanup",
            post(handlers::discord_tokens::cleanup_expired_tokens),
        );

    let app = Router::new()
        .route("/auth/discord/callback", get(handle_discord_callback))
        .route("/auth/discord/exchange", get(handle_discord_exchange))
        .route("/health", get(health_check))
        .route("/health/db", get(database_health_check))
        .nest("/api", api_routes)
        .layer(
            ServiceBuilder::new().layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any),
            ),
        )
        .with_state(state);

    // Validate configuration
    if let Err(errors) = Config::validate() {
        eprintln!("❌ Configuration errors:");
        for error in errors {
            eprintln!("   - {}", error);
        }
        std::process::exit(1);
    }

    // Print configuration summary
    Config::print_summary();

    let bind_address = Config::bind_address();
    let listener = tokio::net::TcpListener::bind(&bind_address).await.unwrap();

    println!("🚀 Server running on {}", Config::server_url());
    axum::serve(listener, app).await.unwrap();
}

#[axum::debug_handler]
async fn handle_discord_callback(
    State(state): State<AppState>,
    Query(params): Query<DiscordCallbackQuery>,
) -> impl IntoResponse {
    if let Some(error) = params.error {
        println!("Discord OAuth error: {}", error);
        return Redirect::to(&Config::oauth_error_redirect("oauth_failed")).into_response();
    }

    if let Some(code) = params.code {
        // Exchange authorization code for access token
        match exchange_code_for_token_and_save_user(code, &state.db_pool).await {
            Ok(user_id) => {
                // Successfully authorized, redirect to home with user ID
                Redirect::to(&Config::oauth_success_redirect(&user_id.to_string())).into_response()
            }
            Err(e) => {
                println!("Token exchange failed: {}", e);
                Redirect::to(&Config::oauth_error_redirect("token_exchange_failed")).into_response()
            }
        }
    } else {
        Redirect::to(&Config::oauth_error_redirect("missing_code")).into_response()
    }
}

#[axum::debug_handler]
async fn handle_discord_exchange(
    State(state): State<AppState>,
    Query(params): Query<DiscordExchangeQuery>,
) -> impl IntoResponse {
    // Exchange authorization code for access token
    match exchange_code_for_token_and_save_user(params.code, &state.db_pool).await {
        Ok(user_id) => {
            // Return JSON with user_id
            Json(serde_json::json!({
                "success": true,
                "user_id": user_id.to_string()
            }))
            .into_response()
        }
        Err(e) => {
            println!("Token exchange failed: {}", e);
            Json(serde_json::json!({
                "success": false,
                "error": "token_exchange_failed",
                "details": e
            }))
            .into_response()
        }
    }
}

async fn exchange_code_for_token_and_save_user(
    code: String,
    pool: &database::DatabasePool,
) -> Result<uuid::Uuid, String> {
    let client_id = Config::discord_client_id();
    let client_secret = Config::discord_client_secret();
    let redirect_uri = Config::discord_redirect_uri();

    let client = reqwest::Client::new();
    let mut params = HashMap::new();
    params.insert("client_id", client_id);
    params.insert("client_secret", client_secret);
    params.insert("grant_type", "authorization_code".to_string());
    params.insert("code", code);
    params.insert("redirect_uri", redirect_uri);

    let client = reqwest::Client::new();
    let response = client
        .post(&Config::discord_token_url())
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if response.status().is_success() {
        let token_response: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("JSON parsing failed: {}", e))?;

        let access_token = token_response
            .get("access_token")
            .and_then(|v| v.as_str())
            .ok_or("No access token in response")?;

        let refresh_token = token_response
            .get("refresh_token")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let token_type = token_response
            .get("token_type")
            .and_then(|v| v.as_str())
            .unwrap_or("Bearer")
            .to_string();

        let scope = token_response
            .get("scope")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let expires_in = token_response.get("expires_in").and_then(|v| v.as_i64());

        // Get user info from Discord API
        let user_info = get_discord_user_info(access_token).await?;

        // Create or update user in database
        let user_id = create_or_update_user_from_discord(&user_info, pool).await?;

        // Save or update Discord token
        let expires_at =
            expires_in.map(|seconds| chrono::Utc::now() + chrono::Duration::seconds(seconds));

        let create_token = shared::database::CreateDiscordToken {
            user_id,
            access_token: access_token.to_string(),
            refresh_token,
            token_type,
            scope,
            expires_at,
        };

        database::queries::upsert_discord_token(pool, create_token)
            .await
            .map_err(|e| format!("Failed to save token: {}", e))?;

        println!("Successfully saved user and token for user_id: {}", user_id);
        Ok(user_id)
    } else {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        Err(format!("Token request failed: {} - {}", status, error_text))
    }
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "discord-oauth-backend"
    }))
}

async fn database_health_check(State(state): State<AppState>) -> Json<serde_json::Value> {
    match db_health_check(&state.db_pool).await {
        Ok(()) => Json(serde_json::json!({
            "status": "healthy",
            "database": "connected"
        })),
        Err(e) => Json(serde_json::json!({
            "status": "unhealthy",
            "database": "disconnected",
            "error": e.to_string()
        })),
    }
}

async fn get_discord_user_info(access_token: &str) -> Result<serde_json::Value, String> {
    let client = reqwest::Client::new();
    let response = client
        .get(&Config::discord_user_api_url())
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
        .map_err(|e| format!("Failed to get user info: {}", e))?;

    if response.status().is_success() {
        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse user info: {}", e))
    } else {
        Err(format!("Failed to get user info: {}", response.status()))
    }
}

async fn create_or_update_user_from_discord(
    user_info: &serde_json::Value,
    pool: &database::DatabasePool,
) -> Result<uuid::Uuid, String> {
    let discord_id = user_info
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or("No Discord ID in user info")?;

    let username = user_info
        .get("username")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown User");

    let global_name = user_info.get("global_name").and_then(|v| v.as_str());

    let display_name = global_name.unwrap_or(username);

    let avatar = user_info
        .get("avatar")
        .and_then(|v| v.as_str())
        .map(|avatar_hash| {
            format!(
                "https://cdn.discordapp.com/avatars/{}/{}.png",
                discord_id, avatar_hash
            )
        });

    // Check if user already exists
    if let Ok(Some(existing_user)) =
        database::queries::get_user_by_discord_id(pool, discord_id).await
    {
        // Update existing user
        let update_user = shared::database::UpdateUser {
            display_name: Some(display_name.to_string()),
            avatar_url: avatar,
            bio: None, // Don't overwrite existing bio
        };

        database::queries::update_user(pool, existing_user.id, update_user)
            .await
            .map_err(|e| format!("Failed to update user: {}", e))?
            .ok_or("User not found after update")?;

        Ok(existing_user.id)
    } else {
        // Create new user
        let create_user = shared::database::CreateUser {
            discord_id: discord_id.to_string(),
            display_name: display_name.to_string(),
            avatar_url: avatar,
            bio: None,
        };

        let new_user = database::queries::create_user(pool, create_user)
            .await
            .map_err(|e| format!("Failed to create user: {}", e))?;

        Ok(new_user.id)
    }
}
