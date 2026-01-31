use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::database::{DatabasePool, queries};
use shared::database::{CreateDiscordToken, UpdateDiscordToken};

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordTokenResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_type: String,
    pub scope: Option<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    // Note: We don't include access_token and refresh_token in responses for security
}

impl From<shared::database::DbDiscordToken> for DiscordTokenResponse {
    fn from(token: shared::database::DbDiscordToken) -> Self {
        Self {
            id: token.id,
            user_id: token.user_id,
            token_type: token.token_type,
            scope: token.scope,
            expires_at: token.expires_at,
            created_at: token.created_at,
            updated_at: token.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateTokenRequest {
    pub user_id: Uuid,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: Option<String>,
    pub scope: Option<String>,
    pub expires_in: Option<i64>, // seconds from now
}

/// Create or update a Discord token for a user (upsert)
pub async fn upsert_discord_token(
    State(state): State<crate::AppState>,
    Json(request): Json<CreateTokenRequest>,
) -> impl IntoResponse {
    let expires_at = request
        .expires_in
        .map(|seconds| chrono::Utc::now() + chrono::Duration::seconds(seconds));

    let create_token = CreateDiscordToken {
        user_id: request.user_id,
        access_token: request.access_token,
        refresh_token: request.refresh_token,
        token_type: request.token_type.unwrap_or_else(|| "Bearer".to_string()),
        scope: request.scope,
        expires_at,
    };

    match queries::upsert_discord_token(&state.db_pool, create_token).await {
        Ok(db_token) => {
            let response_token: DiscordTokenResponse = db_token.into();
            (StatusCode::OK, Json(ApiResponse::success(response_token)))
        }
        Err(e) => {
            let error_msg = if e.to_string().contains("foreign key constraint") {
                "User not found".to_string()
            } else {
                format!("Failed to save Discord token: {}", e)
            };
            (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<DiscordTokenResponse>::error(error_msg)),
            )
        }
    }
}

/// Get Discord token info for a user (without sensitive data)
pub async fn get_discord_token_by_user(
    State(state): State<crate::AppState>,
    Path(user_id): Path<Uuid>,
) -> impl IntoResponse {
    match queries::get_discord_token_by_user_id(&state.db_pool, user_id).await {
        Ok(Some(db_token)) => {
            let response_token: DiscordTokenResponse = db_token.into();
            (StatusCode::OK, Json(ApiResponse::success(response_token)))
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<DiscordTokenResponse>::error(
                "No Discord token found for user".to_string(),
            )),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<DiscordTokenResponse>::error(format!(
                "Database error: {}",
                e
            ))),
        ),
    }
}

/// Update Discord token for a user
pub async fn update_discord_token(
    State(state): State<crate::AppState>,
    Path(user_id): Path<Uuid>,
    Json(update_token): Json<UpdateDiscordToken>,
) -> impl IntoResponse {
    match queries::update_discord_token(&state.db_pool, user_id, update_token).await {
        Ok(Some(db_token)) => {
            let response_token: DiscordTokenResponse = db_token.into();
            (StatusCode::OK, Json(ApiResponse::success(response_token)))
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<DiscordTokenResponse>::error(
                "Discord token not found for user".to_string(),
            )),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<DiscordTokenResponse>::error(format!(
                "Failed to update Discord token: {}",
                e
            ))),
        ),
    }
}

/// Delete Discord token for a user
pub async fn delete_discord_token_by_user(
    State(state): State<crate::AppState>,
    Path(user_id): Path<Uuid>,
) -> impl IntoResponse {
    match queries::delete_discord_token_by_user_id(&state.db_pool, user_id).await {
        Ok(true) => (StatusCode::NO_CONTENT, Json(ApiResponse::<()>::success(()))),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error(
                "No Discord token found for user".to_string(),
            )),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!(
                "Failed to delete Discord token: {}",
                e
            ))),
        ),
    }
}

/// Verify if a Discord token exists and is valid
pub async fn verify_discord_token(
    State(state): State<crate::AppState>,
    Path(user_id): Path<Uuid>,
) -> impl IntoResponse {
    match queries::get_discord_token_by_user_id(&state.db_pool, user_id).await {
        Ok(Some(token)) => {
            // Check if token is expired
            let is_valid = token
                .expires_at
                .map_or(true, |expires| expires > chrono::Utc::now());

            let verification_result = serde_json::json!({
                "user_id": user_id,
                "has_token": true,
                "is_expired": !is_valid,
                "expires_at": token.expires_at
            });

            (
                StatusCode::OK,
                Json(ApiResponse::success(verification_result)),
            )
        }
        Ok(None) => {
            let verification_result = serde_json::json!({
                "user_id": user_id,
                "has_token": false,
                "is_expired": null,
                "expires_at": null
            });

            (
                StatusCode::OK,
                Json(ApiResponse::success(verification_result)),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<serde_json::Value>::error(format!(
                "Database error: {}",
                e
            ))),
        ),
    }
}

/// Cleanup expired tokens (admin endpoint)
pub async fn cleanup_expired_tokens(State(state): State<crate::AppState>) -> impl IntoResponse {
    match queries::cleanup_expired_tokens(&state.db_pool).await {
        Ok(deleted_count) => {
            let result = serde_json::json!({
                "deleted_count": deleted_count,
                "message": format!("Cleaned up {} expired tokens", deleted_count)
            });
            (StatusCode::OK, Json(ApiResponse::success(result)))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<serde_json::Value>::error(format!(
                "Failed to cleanup expired tokens: {}",
                e
            ))),
        ),
    }
}

/// Internal function to get full token data (including sensitive fields)
/// This should only be used internally, never exposed via API
pub async fn get_full_discord_token_by_user_id(
    pool: &DatabasePool,
    user_id: Uuid,
) -> Result<Option<shared::database::DbDiscordToken>, sqlx::Error> {
    queries::get_discord_token_by_user_id(pool, user_id).await
}

/// Internal function to validate access token
/// This should only be used internally for authentication
pub async fn validate_access_token(
    pool: &DatabasePool,
    access_token: &str,
) -> Result<Option<shared::database::DbDiscordToken>, sqlx::Error> {
    let token = queries::get_discord_token_by_access_token(pool, access_token).await?;

    if let Some(ref token) = token {
        // Check if token is expired
        if let Some(expires_at) = token.expires_at {
            if expires_at <= chrono::Utc::now() {
                return Ok(None); // Token is expired
            }
        }
    }

    Ok(token)
}
