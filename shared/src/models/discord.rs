use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::user::User;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordOrg {
    pub id: Uuid,
    pub owner: User,
    pub name: String,
    pub avatar_url: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
