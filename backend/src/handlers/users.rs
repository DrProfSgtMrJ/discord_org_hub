use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::database::{DatabasePool, queries};
use shared::database::{CreateUser, UpdateUser};
use shared::models::User;

#[derive(Deserialize)]
pub struct ListUsersQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

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

/// Create a new user
pub async fn create_user(
    State(state): State<crate::AppState>,
    Json(create_user): Json<CreateUser>,
) -> impl IntoResponse {
    match queries::create_user(&state.db_pool, create_user).await {
        Ok(db_user) => {
            let user: User = db_user.into();
            (StatusCode::CREATED, Json(ApiResponse::success(user)))
        }
        Err(e) => {
            let error_msg = if e.to_string().contains("duplicate key") {
                "User with this Discord ID already exists".to_string()
            } else {
                format!("Failed to create user: {}", e)
            };
            (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<User>::error(error_msg)),
            )
        }
    }
}

/// Get user by ID
pub async fn get_user(
    State(state): State<crate::AppState>,
    Path(user_id): Path<Uuid>,
) -> impl IntoResponse {
    match queries::get_user_by_id(&state.db_pool, user_id).await {
        Ok(Some(db_user)) => {
            let user: User = db_user.into();
            (StatusCode::OK, Json(ApiResponse::success(user)))
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<User>::error("User not found".to_string())),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<User>::error(format!("Database error: {}", e))),
        ),
    }
}

/// Get user by Discord ID
pub async fn get_user_by_discord_id(
    State(state): State<crate::AppState>,
    Path(discord_id): Path<String>,
) -> impl IntoResponse {
    match queries::get_user_by_discord_id(&state.db_pool, &discord_id).await {
        Ok(Some(db_user)) => {
            let user: User = db_user.into();
            (StatusCode::OK, Json(ApiResponse::success(user)))
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<User>::error("User not found".to_string())),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<User>::error(format!("Database error: {}", e))),
        ),
    }
}

/// Update user
pub async fn update_user(
    State(state): State<crate::AppState>,
    Path(user_id): Path<Uuid>,
    Json(update_user): Json<UpdateUser>,
) -> impl IntoResponse {
    match queries::update_user(&state.db_pool, user_id, update_user).await {
        Ok(Some(db_user)) => {
            let user: User = db_user.into();
            (StatusCode::OK, Json(ApiResponse::success(user)))
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<User>::error("User not found".to_string())),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<User>::error(format!(
                "Failed to update user: {}",
                e
            ))),
        ),
    }
}

/// Delete user
pub async fn delete_user(
    State(state): State<crate::AppState>,
    Path(user_id): Path<Uuid>,
) -> impl IntoResponse {
    match queries::delete_user(&state.db_pool, user_id).await {
        Ok(true) => (StatusCode::NO_CONTENT, Json(ApiResponse::<()>::success(()))),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("User not found".to_string())),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!(
                "Failed to delete user: {}",
                e
            ))),
        ),
    }
}

/// List users with pagination
pub async fn list_users(
    State(state): State<crate::AppState>,
    Query(params): Query<ListUsersQuery>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(50).min(100); // Max 100 items per page
    let offset = params.offset.unwrap_or(0).max(0); // Ensure non-negative

    match queries::list_users(&state.db_pool, limit, offset).await {
        Ok(db_users) => {
            let users: Vec<User> = db_users.into_iter().map(|db_user| db_user.into()).collect();
            (StatusCode::OK, Json(ApiResponse::success(users)))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<Vec<User>>::error(format!(
                "Failed to list users: {}",
                e
            ))),
        ),
    }
}

#[derive(Serialize)]
pub struct UserStats {
    pub total_users: i64,
    pub users_with_bio: i64,
    pub users_with_avatar: i64,
}

/// Get user statistics
pub async fn get_user_stats(State(state): State<crate::AppState>) -> impl IntoResponse {
    // Note: This would require additional query functions
    // For now, we'll return a simple count
    match sqlx::query_scalar!("SELECT COUNT(*) FROM users")
        .fetch_one(&*state.db_pool)
        .await
    {
        Ok(total) => {
            let stats = UserStats {
                total_users: total.unwrap_or(0),
                users_with_bio: 0,    // Would need additional queries
                users_with_avatar: 0, // Would need additional queries
            };
            (StatusCode::OK, Json(ApiResponse::success(stats)))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<UserStats>::error(format!(
                "Failed to get user stats: {}",
                e
            ))),
        ),
    }
}
