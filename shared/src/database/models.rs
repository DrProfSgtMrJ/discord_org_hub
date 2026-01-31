use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "database")]
use sqlx::FromRow;

use crate::models::{DiscordOrg, Member, MemberStatus, User};

// Database model for users table
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(FromRow))]
pub struct DbUser {
    pub id: Uuid,
    pub discord_id: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<DbUser> for User {
    fn from(db_user: DbUser) -> Self {
        User {
            id: db_user.id,
            discord_id: db_user.discord_id,
            display_name: db_user.display_name,
            avatar_url: db_user.avatar_url,
            bio: db_user.bio,
            created_at: db_user.created_at,
            updated_at: db_user.updated_at,
        }
    }
}

impl From<User> for DbUser {
    fn from(user: User) -> Self {
        DbUser {
            id: user.id,
            discord_id: user.discord_id,
            display_name: user.display_name,
            avatar_url: user.avatar_url,
            bio: user.bio,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

// Database model for discord_orgs table
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(FromRow))]
pub struct DbDiscordOrg {
    pub id: Uuid,
    pub owner_id: Uuid, // Foreign key to users table
    pub name: String,
    pub avatar_url: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Database model for members table
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(FromRow))]
pub struct DbMember {
    pub id: Uuid,
    pub user_id: Uuid,        // Foreign key to users table
    pub discord_org_id: Uuid, // Foreign key to discord_orgs table
    pub status: String,       // MemberStatus as string for database storage
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<MemberStatus> for String {
    fn from(status: MemberStatus) -> Self {
        match status {
            MemberStatus::Spectating => "spectating".to_string(),
            MemberStatus::Playing => "playing".to_string(),
            MemberStatus::Banned => "banned".to_string(),
        }
    }
}

impl TryFrom<String> for MemberStatus {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "spectating" => Ok(MemberStatus::Spectating),
            "playing" => Ok(MemberStatus::Playing),
            "banned" => Ok(MemberStatus::Banned),
            _ => Err(format!("Invalid member status: {}", value)),
        }
    }
}

// Join query result for getting a member with user and org data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(FromRow))]
pub struct DbMemberWithRelations {
    // Member fields
    pub id: Uuid,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // User fields (prefixed with user_)
    pub user_id: Uuid,
    pub user_discord_id: String,
    pub user_display_name: String,
    pub user_avatar_url: Option<String>,
    pub user_bio: Option<String>,
    pub user_created_at: DateTime<Utc>,
    pub user_updated_at: DateTime<Utc>,

    // DiscordOrg fields (prefixed with org_)
    pub org_id: Uuid,
    pub org_owner_id: Uuid,
    pub org_name: String,
    pub org_avatar_url: Option<String>,
    pub org_description: Option<String>,
    pub org_created_at: DateTime<Utc>,
    pub org_updated_at: DateTime<Utc>,

    // Owner fields (prefixed with owner_)
    pub owner_discord_id: String,
    pub owner_display_name: String,
    pub owner_avatar_url: Option<String>,
    pub owner_bio: Option<String>,
    pub owner_created_at: DateTime<Utc>,
    pub owner_updated_at: DateTime<Utc>,
}

impl TryFrom<DbMemberWithRelations> for Member {
    type Error = String;

    fn try_from(db_member: DbMemberWithRelations) -> Result<Self, Self::Error> {
        let user = User {
            id: db_member.user_id,
            discord_id: db_member.user_discord_id,
            display_name: db_member.user_display_name,
            avatar_url: db_member.user_avatar_url,
            bio: db_member.user_bio,
            created_at: db_member.user_created_at,
            updated_at: db_member.user_updated_at,
        };

        let owner = User {
            id: db_member.org_owner_id,
            discord_id: db_member.owner_discord_id,
            display_name: db_member.owner_display_name,
            avatar_url: db_member.owner_avatar_url,
            bio: db_member.owner_bio,
            created_at: db_member.owner_created_at,
            updated_at: db_member.owner_updated_at,
        };

        let discord_org = DiscordOrg {
            id: db_member.org_id,
            owner,
            name: db_member.org_name,
            avatar_url: db_member.org_avatar_url,
            description: db_member.org_description,
            created_at: db_member.org_created_at,
            updated_at: db_member.org_updated_at,
        };

        let status = MemberStatus::try_from(db_member.status)?;

        Ok(Member {
            id: db_member.id,
            user,
            discord_org,
            status,
            created_at: db_member.created_at,
            updated_at: db_member.updated_at,
        })
    }
}

// Input models for creating new records
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUser {
    pub discord_id: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDiscordOrg {
    pub owner_id: Uuid,
    pub name: String,
    pub avatar_url: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMember {
    pub user_id: Uuid,
    pub discord_org_id: Uuid,
    pub status: MemberStatus,
}

// Update models for modifying existing records
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUser {
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDiscordOrg {
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMember {
    pub status: Option<MemberStatus>,
}

// Database model for discord_tokens table
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(FromRow))]
pub struct DbDiscordToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub scope: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Input model for creating new discord tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDiscordToken {
    pub user_id: Uuid,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub scope: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

// Update model for modifying existing discord tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDiscordToken {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub token_type: Option<String>,
    pub scope: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}
