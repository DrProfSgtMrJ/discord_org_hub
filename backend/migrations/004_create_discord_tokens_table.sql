-- Create discord_tokens table to store authentication tokens
CREATE TABLE IF NOT EXISTS discord_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    access_token TEXT NOT NULL,
    refresh_token TEXT,
    token_type VARCHAR(50) NOT NULL DEFAULT 'Bearer',
    scope TEXT,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Ensure a user can only have one active token
    UNIQUE(user_id)
);

-- Create index on user_id for faster lookups
CREATE INDEX IF NOT EXISTS idx_discord_tokens_user_id ON discord_tokens(user_id);

-- Create index on access_token for token validation
CREATE INDEX IF NOT EXISTS idx_discord_tokens_access_token ON discord_tokens(access_token);

-- Create index on expires_at for cleanup of expired tokens
CREATE INDEX IF NOT EXISTS idx_discord_tokens_expires_at ON discord_tokens(expires_at);

-- Create trigger to automatically update updated_at on discord_tokens table
CREATE TRIGGER update_discord_tokens_updated_at BEFORE UPDATE ON discord_tokens
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Add comment explaining the table
COMMENT ON TABLE discord_tokens IS 'Stores Discord OAuth2 access tokens for authenticated users';
COMMENT ON COLUMN discord_tokens.access_token IS 'Discord OAuth2 access token for API calls';
COMMENT ON COLUMN discord_tokens.refresh_token IS 'Discord OAuth2 refresh token for token renewal';
COMMENT ON COLUMN discord_tokens.token_type IS 'Token type (usually Bearer)';
COMMENT ON COLUMN discord_tokens.scope IS 'OAuth2 scopes granted to this token';
COMMENT ON COLUMN discord_tokens.expires_at IS 'When the access token expires (NULL for non-expiring tokens)';
