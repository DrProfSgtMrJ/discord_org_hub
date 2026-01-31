-- Create member_status enum
CREATE TYPE member_status AS ENUM ('spectating', 'playing', 'banned');

-- Create members table
CREATE TABLE IF NOT EXISTS members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    discord_org_id UUID NOT NULL REFERENCES discord_orgs(id) ON DELETE CASCADE,
    status member_status NOT NULL DEFAULT 'spectating',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Ensure a user can only be a member of an org once
    UNIQUE(user_id, discord_org_id)
);

-- Create index on user_id for faster lookups
CREATE INDEX IF NOT EXISTS idx_members_user_id ON members(user_id);

-- Create index on discord_org_id for faster lookups
CREATE INDEX IF NOT EXISTS idx_members_discord_org_id ON members(discord_org_id);

-- Create index on status for filtering
CREATE INDEX IF NOT EXISTS idx_members_status ON members(status);

-- Create composite index for common queries
CREATE INDEX IF NOT EXISTS idx_members_org_status ON members(discord_org_id, status);

-- Create trigger to automatically update updated_at on members table
CREATE TRIGGER update_members_updated_at BEFORE UPDATE ON members
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
