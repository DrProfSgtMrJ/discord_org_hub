use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::discord::DiscordOrg;
use crate::models::user::User;

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum MemberStatus {
    Spectating,
    Playing,
    Banned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Member {
    pub id: Uuid,
    pub user: User,
    pub discord_org: DiscordOrg,
    pub status: MemberStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
