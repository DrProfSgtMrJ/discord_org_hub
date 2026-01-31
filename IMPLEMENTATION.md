# Discord OAuth Implementation Summary

## Overview

This document summarizes the Discord OAuth2 authentication system implemented for the Discord Organization Hub. The implementation provides a complete sign-up and login flow using Discord's OAuth2 service.

## Architecture

### Frontend (Dioxus/Rust/WASM)
- **Location**: `/frontend/src/`
- **Framework**: Dioxus with web target
- **Key Components**:
  - `views/home.rs` - Main home page with authentication UI
  - `services/discord.rs` - Discord OAuth service
  - `services/api.rs` - Backend API communication (prepared for future use)

### Backend (Axum/Rust)
- **Location**: `/backend/src/`
- **Framework**: Axum web server
- **Database**: PostgreSQL with SQLx
- **Key Components**:
  - OAuth callback handler (`/auth/discord/callback`)
  - User management API endpoints
  - Discord token storage and management

## Implementation Details

### 1. Frontend Authentication Flow

#### Home Page (`frontend/src/views/home.rs`)
- Displays different UI based on authentication state
- For unauthenticated users: Shows welcome content and Discord login button
- For authenticated users: Shows dashboard with user info and logout option
- Handles OAuth callback parameters and status messages
- Persists authentication state in localStorage

#### Discord Service (`frontend/src/services/discord.rs`)
- `start_oauth_flow()` - Redirects user to Discord OAuth authorization
- `parse_callback_params()` - Parses URL parameters from OAuth callback
- `store_auth_state()` / `get_stored_auth_state()` - localStorage management
- `clear_url_params()` - Cleans up URL after OAuth flow

#### Key Features:
- **Client ID**: `1466997290819649619`
- **Redirect URI**: `http://localhost:8080/auth/discord/callback`
- **Scopes**: `identify` (gets user ID, username, avatar)
- **Flow Type**: Authorization code flow (secure)
- **State Persistence**: localStorage for login persistence

### 2. Backend OAuth Implementation

#### OAuth Callback Handler (`backend/src/main.rs`)
```rust
async fn handle_discord_callback(
    State(state): State<AppState>,
    Query(params): Query<DiscordCallbackQuery>,
) -> impl IntoResponse
```

**Process Flow**:
1. Receives authorization code from Discord
2. Exchanges code for access token via Discord API
3. Fetches user information from Discord `/users/@me` endpoint
4. Creates or updates user in database
5. Stores Discord token for future API calls
6. Redirects user back to frontend (port 8081) with success/error status

#### User Data Extraction
From Discord's `/users/@me` API response:
- **ID**: Discord user ID (stored as `discord_id`)
- **Username**: Discord username
- **Global Name**: Display name (preferred over username)
- **Avatar**: Avatar hash (converted to CDN URL)

#### Database Schema

**Users Table** (`migrations/001_create_users_table.sql`):
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    discord_id VARCHAR(255) NOT NULL UNIQUE,
    display_name VARCHAR(255) NOT NULL,
    avatar_url TEXT,
    bio TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**Discord Tokens Table** (`migrations/004_create_discord_tokens_table.sql`):
```sql
CREATE TABLE discord_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    access_token TEXT NOT NULL,
    refresh_token TEXT,
    token_type VARCHAR(50) NOT NULL DEFAULT 'Bearer',
    scope TEXT,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id)
);
```

### 3. API Endpoints

#### Authentication
- `GET /auth/discord/callback` - Discord OAuth callback handler

#### Users
- `GET /api/users` - List all users
- `POST /api/users` - Create new user
- `GET /api/users/:id` - Get user by UUID
- `GET /api/users/discord/:discord_id` - Get user by Discord ID
- `PUT /api/users/:id` - Update user
- `DELETE /api/users/:id` - Delete user

#### Discord Tokens
- `POST /api/discord-tokens` - Create/update token
- `GET /api/discord-tokens/user/:user_id` - Get token by user
- `PUT /api/discord-tokens/user/:user_id` - Update token
- `DELETE /api/discord-tokens/user/:user_id` - Delete token
- `GET /api/discord-tokens/verify/:user_id` - Verify token validity

#### Health Checks
- `GET /health` - API health status
- `GET /health/db` - Database connectivity check

### 4. Security Features

- **CORS**: Configured for localhost development
- **Authorization Code Flow**: More secure than implicit flow
- **Token Storage**: Secure database storage with expiration
- **Input Validation**: Proper error handling for OAuth failures
- **State Management**: Frontend state persistence without exposing tokens

## Environment Configuration

Required environment variables in `backend/.env`:

```env
# Database
DATABASE_URL=postgresql://username:password@localhost/discord_org_hub

# Server
HOST=127.0.0.1
PORT=8080

# Frontend Configuration
FRONTEND_URL=http://localhost:8081

# Discord OAuth
DISCORD_CLIENT_ID=1466997290819649619
DISCORD_CLIENT_SECRET=your_secret_here
DISCORD_REDIRECT_URI=http://localhost:8081/auth/discord/callback
```

## CSS Styling

### Key UI Components
- **Discord Login Button**: Discord-branded styling (#5865f2)
- **User Avatar**: Rounded profile picture display
- **Auth Status Messages**: Success/error feedback
- **Authenticated Dashboard**: Bot download and quick actions
- **Responsive Design**: Mobile-friendly layout
- **Loading States**: Visual feedback during operations

### Theme
- **Dark Theme**: Primary background #0f1116
- **Accent Colors**: Discord blue (#5865f2), success green, error red
- **Typography**: Segoe UI font stack
- **Gradients**: Used for buttons and titles

## Development Workflow

1. **Start Backend**: `cargo run` in `/backend` (runs on port 8080)
2. **Start Frontend**: `dx serve --port 8081` in `/frontend` (runs on port 8081)
3. **Access App**: Navigate to `http://localhost:8081`
4. **Database**: Ensure PostgreSQL is running with migrations applied
5. **OAuth Flow**: Discord redirects to backend (8080) which then redirects to frontend (8081)

## Future Enhancements

### Short Term
- Implement session management and token refresh
- Add user profile editing capabilities
- Enhanced bot download tracking and status
- Error boundary improvements and better error messages

### Medium Term
- Refresh token handling
- Rate limiting
- User roles and permissions
- Server/guild integration

### Long Term
- Multi-server support
- Advanced analytics
- Bot integration features
- Production deployment pipeline

## Testing Strategy

### Manual Testing
- OAuth flow from start to finish
- Error handling (declined authorization, network issues)
- State persistence across browser sessions
- Mobile responsiveness

### API Testing
- Health endpoints verification
- Database operations
- Error responses
- CORS functionality

### Database Testing
- Migration scripts
- User creation/updates
- Token storage and cleanup
- Foreign key constraints

## Known Limitations

1. **Development Only**: Configured for localhost only
2. **Basic Error Handling**: Could be more granular
3. **No Session Timeout**: Relies on localStorage persistence
4. **Single Server**: No multi-guild support yet
5. **No Refresh Logic**: Tokens expire without automatic renewal
6. **Port Configuration**: Frontend and backend ports are hardcoded for development

## Dependencies

### Frontend
- **dioxus**: 0.7.1 (UI framework)
- **web-sys**: 0.3 (Browser APIs)
- **serde**: JSON serialization
- **wasm-bindgen**: JavaScript interop

### Backend
- **axum**: 0.7 (Web framework)
- **sqlx**: 0.7 (Database driver)
- **reqwest**: 0.11 (HTTP client)
- **tower-http**: 0.5 (Middleware)
- **tokio**: 1.40 (Async runtime)

This implementation provides a solid foundation for Discord-based authentication and can be extended with additional features as needed.
