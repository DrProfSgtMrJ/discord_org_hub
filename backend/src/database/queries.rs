use sqlx::Type;
use uuid::Uuid;

use shared::database::{
    CreateDiscordOrg, CreateDiscordToken, CreateMember, CreateUser, DbDiscordOrg, DbDiscordToken,
    DbMember, DbMemberWithRelations, DbUser, UpdateDiscordOrg, UpdateDiscordToken, UpdateMember,
    UpdateUser,
};
use shared::models::MemberStatus;

use crate::database::DatabasePool;

// Custom type for PostgreSQL enum
#[derive(Debug, Clone, Type)]
#[sqlx(type_name = "member_status", rename_all = "lowercase")]
enum PgMemberStatus {
    Spectating,
    Playing,
    Banned,
}

impl From<MemberStatus> for PgMemberStatus {
    fn from(status: MemberStatus) -> Self {
        match status {
            MemberStatus::Spectating => PgMemberStatus::Spectating,
            MemberStatus::Playing => PgMemberStatus::Playing,
            MemberStatus::Banned => PgMemberStatus::Banned,
        }
    }
}

impl From<PgMemberStatus> for MemberStatus {
    fn from(status: PgMemberStatus) -> Self {
        match status {
            PgMemberStatus::Spectating => MemberStatus::Spectating,
            PgMemberStatus::Playing => MemberStatus::Playing,
            PgMemberStatus::Banned => MemberStatus::Banned,
        }
    }
}

// User queries
pub async fn create_user(
    pool: &DatabasePool,
    create_user: CreateUser,
) -> Result<DbUser, sqlx::Error> {
    let user = sqlx::query_as!(
        DbUser,
        r#"
        INSERT INTO users (discord_id, display_name, avatar_url, bio)
        VALUES ($1, $2, $3, $4)
        RETURNING id, discord_id, display_name, avatar_url, bio, created_at, updated_at
        "#,
        create_user.discord_id,
        create_user.display_name,
        create_user.avatar_url,
        create_user.bio
    )
    .fetch_one(pool)
    .await?;

    Ok(user)
}

pub async fn get_user_by_id(
    pool: &DatabasePool,
    user_id: Uuid,
) -> Result<Option<DbUser>, sqlx::Error> {
    let user = sqlx::query_as!(
        DbUser,
        "SELECT id, discord_id, display_name, avatar_url, bio, created_at, updated_at FROM users WHERE id = $1",
        user_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

pub async fn get_user_by_discord_id(
    pool: &DatabasePool,
    discord_id: &str,
) -> Result<Option<DbUser>, sqlx::Error> {
    let user = sqlx::query_as!(
        DbUser,
        "SELECT id, discord_id, display_name, avatar_url, bio, created_at, updated_at FROM users WHERE discord_id = $1",
        discord_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

pub async fn update_user(
    pool: &DatabasePool,
    user_id: Uuid,
    update_user: UpdateUser,
) -> Result<Option<DbUser>, sqlx::Error> {
    let user = sqlx::query_as!(
        DbUser,
        r#"
        UPDATE users
        SET
            display_name = COALESCE($2, display_name),
            avatar_url = COALESCE($3, avatar_url),
            bio = COALESCE($4, bio),
            updated_at = NOW()
        WHERE id = $1
        RETURNING id, discord_id, display_name, avatar_url, bio, created_at, updated_at
        "#,
        user_id,
        update_user.display_name,
        update_user.avatar_url,
        update_user.bio
    )
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

pub async fn delete_user(pool: &DatabasePool, user_id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn list_users(
    pool: &DatabasePool,
    limit: i64,
    offset: i64,
) -> Result<Vec<DbUser>, sqlx::Error> {
    let users = sqlx::query_as!(
        DbUser,
        "SELECT id, discord_id, display_name, avatar_url, bio, created_at, updated_at FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        limit,
        offset
    )
    .fetch_all(pool)
    .await?;

    Ok(users)
}

// Discord org queries
pub async fn create_discord_org(
    pool: &DatabasePool,
    create_org: CreateDiscordOrg,
) -> Result<DbDiscordOrg, sqlx::Error> {
    let org = sqlx::query_as!(
        DbDiscordOrg,
        r#"
        INSERT INTO discord_orgs (owner_id, name, avatar_url, description)
        VALUES ($1, $2, $3, $4)
        RETURNING id, owner_id, name, avatar_url, description, created_at, updated_at
        "#,
        create_org.owner_id,
        create_org.name,
        create_org.avatar_url,
        create_org.description
    )
    .fetch_one(pool)
    .await?;

    Ok(org)
}

pub async fn get_discord_org_by_id(
    pool: &DatabasePool,
    org_id: Uuid,
) -> Result<Option<DbDiscordOrg>, sqlx::Error> {
    let org = sqlx::query_as!(
        DbDiscordOrg,
        "SELECT id, owner_id, name, avatar_url, description, created_at, updated_at FROM discord_orgs WHERE id = $1",
        org_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(org)
}

pub async fn get_discord_orgs_by_owner(
    pool: &DatabasePool,
    owner_id: Uuid,
) -> Result<Vec<DbDiscordOrg>, sqlx::Error> {
    let orgs = sqlx::query_as!(
        DbDiscordOrg,
        "SELECT id, owner_id, name, avatar_url, description, created_at, updated_at FROM discord_orgs WHERE owner_id = $1 ORDER BY created_at DESC",
        owner_id
    )
    .fetch_all(pool)
    .await?;

    Ok(orgs)
}

pub async fn update_discord_org(
    pool: &DatabasePool,
    org_id: Uuid,
    update_org: UpdateDiscordOrg,
) -> Result<Option<DbDiscordOrg>, sqlx::Error> {
    let org = sqlx::query_as!(
        DbDiscordOrg,
        r#"
        UPDATE discord_orgs
        SET
            name = COALESCE($2, name),
            avatar_url = COALESCE($3, avatar_url),
            description = COALESCE($4, description),
            updated_at = NOW()
        WHERE id = $1
        RETURNING id, owner_id, name, avatar_url, description, created_at, updated_at
        "#,
        org_id,
        update_org.name,
        update_org.avatar_url,
        update_org.description
    )
    .fetch_optional(pool)
    .await?;

    Ok(org)
}

pub async fn delete_discord_org(pool: &DatabasePool, org_id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!("DELETE FROM discord_orgs WHERE id = $1", org_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn list_discord_orgs(
    pool: &DatabasePool,
    limit: i64,
    offset: i64,
) -> Result<Vec<DbDiscordOrg>, sqlx::Error> {
    let orgs = sqlx::query_as!(
        DbDiscordOrg,
        "SELECT id, owner_id, name, avatar_url, description, created_at, updated_at FROM discord_orgs ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        limit,
        offset
    )
    .fetch_all(pool)
    .await?;

    Ok(orgs)
}

// Member queries
pub async fn create_member(
    pool: &DatabasePool,
    create_member: CreateMember,
) -> Result<DbMember, sqlx::Error> {
    let pg_status: PgMemberStatus = create_member.status.into();

    let member = sqlx::query_as!(
        DbMember,
        r#"
        INSERT INTO members (user_id, discord_org_id, status)
        VALUES ($1, $2, $3)
        RETURNING id, user_id, discord_org_id, status as "status: String", created_at, updated_at
        "#,
        create_member.user_id,
        create_member.discord_org_id,
        pg_status as PgMemberStatus
    )
    .fetch_one(pool)
    .await?;

    Ok(member)
}

pub async fn get_member_by_id(
    pool: &DatabasePool,
    member_id: Uuid,
) -> Result<Option<DbMember>, sqlx::Error> {
    let member = sqlx::query_as!(
        DbMember,
        r#"SELECT id, user_id, discord_org_id, status as "status: String", created_at, updated_at FROM members WHERE id = $1"#,
        member_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(member)
}

pub async fn get_member_with_relations(
    pool: &DatabasePool,
    member_id: Uuid,
) -> Result<Option<DbMemberWithRelations>, sqlx::Error> {
    let member = sqlx::query_as!(
        DbMemberWithRelations,
        r#"
        SELECT
            m.id,
            m.status as "status: String",
            m.created_at,
            m.updated_at,

            u.id as user_id,
            u.discord_id as user_discord_id,
            u.display_name as user_display_name,
            u.avatar_url as user_avatar_url,
            u.bio as user_bio,
            u.created_at as user_created_at,
            u.updated_at as user_updated_at,

            o.id as org_id,
            o.owner_id as org_owner_id,
            o.name as org_name,
            o.avatar_url as org_avatar_url,
            o.description as org_description,
            o.created_at as org_created_at,
            o.updated_at as org_updated_at,

            owner.discord_id as owner_discord_id,
            owner.display_name as owner_display_name,
            owner.avatar_url as owner_avatar_url,
            owner.bio as owner_bio,
            owner.created_at as owner_created_at,
            owner.updated_at as owner_updated_at
        FROM members m
        JOIN users u ON m.user_id = u.id
        JOIN discord_orgs o ON m.discord_org_id = o.id
        JOIN users owner ON o.owner_id = owner.id
        WHERE m.id = $1
        "#,
        member_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(member)
}

pub async fn get_members_by_org(
    pool: &DatabasePool,
    org_id: Uuid,
) -> Result<Vec<DbMemberWithRelations>, sqlx::Error> {
    let members = sqlx::query_as!(
        DbMemberWithRelations,
        r#"
        SELECT
            m.id,
            m.status as "status: String",
            m.created_at,
            m.updated_at,

            u.id as user_id,
            u.discord_id as user_discord_id,
            u.display_name as user_display_name,
            u.avatar_url as user_avatar_url,
            u.bio as user_bio,
            u.created_at as user_created_at,
            u.updated_at as user_updated_at,

            o.id as org_id,
            o.owner_id as org_owner_id,
            o.name as org_name,
            o.avatar_url as org_avatar_url,
            o.description as org_description,
            o.created_at as org_created_at,
            o.updated_at as org_updated_at,

            owner.discord_id as owner_discord_id,
            owner.display_name as owner_display_name,
            owner.avatar_url as owner_avatar_url,
            owner.bio as owner_bio,
            owner.created_at as owner_created_at,
            owner.updated_at as owner_updated_at
        FROM members m
        JOIN users u ON m.user_id = u.id
        JOIN discord_orgs o ON m.discord_org_id = o.id
        JOIN users owner ON o.owner_id = owner.id
        WHERE m.discord_org_id = $1
        ORDER BY m.created_at ASC
        "#,
        org_id
    )
    .fetch_all(pool)
    .await?;

    Ok(members)
}

pub async fn get_members_by_user(
    pool: &DatabasePool,
    user_id: Uuid,
) -> Result<Vec<DbMemberWithRelations>, sqlx::Error> {
    let members = sqlx::query_as!(
        DbMemberWithRelations,
        r#"
        SELECT
            m.id,
            m.status as "status: String",
            m.created_at,
            m.updated_at,

            u.id as user_id,
            u.discord_id as user_discord_id,
            u.display_name as user_display_name,
            u.avatar_url as user_avatar_url,
            u.bio as user_bio,
            u.created_at as user_created_at,
            u.updated_at as user_updated_at,

            o.id as org_id,
            o.owner_id as org_owner_id,
            o.name as org_name,
            o.avatar_url as org_avatar_url,
            o.description as org_description,
            o.created_at as org_created_at,
            o.updated_at as org_updated_at,

            owner.discord_id as owner_discord_id,
            owner.display_name as owner_display_name,
            owner.avatar_url as owner_avatar_url,
            owner.bio as owner_bio,
            owner.created_at as owner_created_at,
            owner.updated_at as owner_updated_at
        FROM members m
        JOIN users u ON m.user_id = u.id
        JOIN discord_orgs o ON m.discord_org_id = o.id
        JOIN users owner ON o.owner_id = owner.id
        WHERE m.user_id = $1
        ORDER BY m.created_at ASC
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(members)
}

pub async fn update_member(
    pool: &DatabasePool,
    member_id: Uuid,
    update_member: UpdateMember,
) -> Result<Option<DbMember>, sqlx::Error> {
    let pg_status = update_member.status.map(|s| -> PgMemberStatus { s.into() });

    let member = sqlx::query_as!(
        DbMember,
        r#"
        UPDATE members
        SET
            status = COALESCE($2, status),
            updated_at = NOW()
        WHERE id = $1
        RETURNING id, user_id, discord_org_id, status as "status: String", created_at, updated_at
        "#,
        member_id,
        pg_status as Option<PgMemberStatus>
    )
    .fetch_optional(pool)
    .await?;

    Ok(member)
}

pub async fn delete_member(pool: &DatabasePool, member_id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!("DELETE FROM members WHERE id = $1", member_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn get_member_by_user_and_org(
    pool: &DatabasePool,
    user_id: Uuid,
    org_id: Uuid,
) -> Result<Option<DbMember>, sqlx::Error> {
    let member = sqlx::query_as!(
        DbMember,
        r#"SELECT id, user_id, discord_org_id, status as "status: String", created_at, updated_at FROM members WHERE user_id = $1 AND discord_org_id = $2"#,
        user_id,
        org_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(member)
}

// Utility functions
pub async fn count_members_by_org(pool: &DatabasePool, org_id: Uuid) -> Result<i64, sqlx::Error> {
    let count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM members WHERE discord_org_id = $1",
        org_id
    )
    .fetch_one(pool)
    .await?;

    Ok(count.unwrap_or(0))
}

pub async fn count_members_by_status(
    pool: &DatabasePool,
    org_id: Uuid,
    status: MemberStatus,
) -> Result<i64, sqlx::Error> {
    let pg_status: PgMemberStatus = status.into();
    let count: Option<i64> = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM members WHERE discord_org_id = $1 AND status = $2",
        org_id,
        pg_status as PgMemberStatus
    )
    .fetch_one(pool)
    .await?;

    Ok(count.unwrap_or(0))
}

// Discord token queries
pub async fn create_discord_token(
    pool: &DatabasePool,
    create_token: CreateDiscordToken,
) -> Result<DbDiscordToken, sqlx::Error> {
    let token = sqlx::query_as!(
        DbDiscordToken,
        r#"
        INSERT INTO discord_tokens (user_id, access_token, refresh_token, token_type, scope, expires_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, user_id, access_token, refresh_token, token_type, scope, expires_at, created_at, updated_at
        "#,
        create_token.user_id,
        create_token.access_token,
        create_token.refresh_token,
        create_token.token_type,
        create_token.scope,
        create_token.expires_at
    )
    .fetch_one(pool)
    .await?;

    Ok(token)
}

pub async fn get_discord_token_by_user_id(
    pool: &DatabasePool,
    user_id: Uuid,
) -> Result<Option<DbDiscordToken>, sqlx::Error> {
    let token = sqlx::query_as!(
        DbDiscordToken,
        "SELECT id, user_id, access_token, refresh_token, token_type, scope, expires_at, created_at, updated_at FROM discord_tokens WHERE user_id = $1",
        user_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(token)
}

pub async fn get_discord_token_by_access_token(
    pool: &DatabasePool,
    access_token: &str,
) -> Result<Option<DbDiscordToken>, sqlx::Error> {
    let token = sqlx::query_as!(
        DbDiscordToken,
        "SELECT id, user_id, access_token, refresh_token, token_type, scope, expires_at, created_at, updated_at FROM discord_tokens WHERE access_token = $1",
        access_token
    )
    .fetch_optional(pool)
    .await?;

    Ok(token)
}

pub async fn update_discord_token(
    pool: &DatabasePool,
    user_id: Uuid,
    update_token: UpdateDiscordToken,
) -> Result<Option<DbDiscordToken>, sqlx::Error> {
    let token = sqlx::query_as!(
        DbDiscordToken,
        r#"
        UPDATE discord_tokens
        SET
            access_token = COALESCE($2, access_token),
            refresh_token = COALESCE($3, refresh_token),
            token_type = COALESCE($4, token_type),
            scope = COALESCE($5, scope),
            expires_at = COALESCE($6, expires_at),
            updated_at = NOW()
        WHERE user_id = $1
        RETURNING id, user_id, access_token, refresh_token, token_type, scope, expires_at, created_at, updated_at
        "#,
        user_id,
        update_token.access_token,
        update_token.refresh_token,
        update_token.token_type,
        update_token.scope,
        update_token.expires_at
    )
    .fetch_optional(pool)
    .await?;

    Ok(token)
}

pub async fn delete_discord_token_by_user_id(
    pool: &DatabasePool,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!("DELETE FROM discord_tokens WHERE user_id = $1", user_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn delete_discord_token_by_access_token(
    pool: &DatabasePool,
    access_token: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        "DELETE FROM discord_tokens WHERE access_token = $1",
        access_token
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn cleanup_expired_tokens(pool: &DatabasePool) -> Result<i64, sqlx::Error> {
    let result = sqlx::query!(
        "DELETE FROM discord_tokens WHERE expires_at IS NOT NULL AND expires_at < NOW()"
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() as i64)
}

pub async fn upsert_discord_token(
    pool: &DatabasePool,
    create_token: CreateDiscordToken,
) -> Result<DbDiscordToken, sqlx::Error> {
    let token = sqlx::query_as!(
        DbDiscordToken,
        r#"
        INSERT INTO discord_tokens (user_id, access_token, refresh_token, token_type, scope, expires_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (user_id)
        DO UPDATE SET
            access_token = EXCLUDED.access_token,
            refresh_token = EXCLUDED.refresh_token,
            token_type = EXCLUDED.token_type,
            scope = EXCLUDED.scope,
            expires_at = EXCLUDED.expires_at,
            updated_at = NOW()
        RETURNING id, user_id, access_token, refresh_token, token_type, scope, expires_at, created_at, updated_at
        "#,
        create_token.user_id,
        create_token.access_token,
        create_token.refresh_token,
        create_token.token_type,
        create_token.scope,
        create_token.expires_at
    )
    .fetch_one(pool)
    .await?;

    Ok(token)
}
