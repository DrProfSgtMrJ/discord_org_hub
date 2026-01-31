-- Create discord_orgs table
CREATE TABLE IF NOT EXISTS discord_orgs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    avatar_url TEXT,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create index on owner_id for faster lookups
CREATE INDEX IF NOT EXISTS idx_discord_orgs_owner_id ON discord_orgs(owner_id);

-- Create index on name for search functionality
CREATE INDEX IF NOT EXISTS idx_discord_orgs_name ON discord_orgs(name);

-- Create trigger to automatically update updated_at on discord_orgs table
CREATE TRIGGER update_discord_orgs_updated_at BEFORE UPDATE ON discord_orgs
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
