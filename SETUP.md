# Discord Organization Hub - Setup Instructions

## Prerequisites

- Rust (latest stable version)
- PostgreSQL database
- Discord Application (for OAuth)

## Environment Variables

Create a `.env` file in the `backend` directory with the following variables:

```env
# Database Configuration
DATABASE_URL=postgresql://username:password@localhost/discord_org_hub

# Server Configuration
HOST=127.0.0.1
PORT=8080

# Frontend Configuration
FRONTEND_URL=http://localhost:8081

# Discord OAuth Configuration
DISCORD_CLIENT_ID=1466997290819649619
DISCORD_CLIENT_SECRET=your_discord_client_secret_here
DISCORD_REDIRECT_URI=http://localhost:8081/auth/discord/callback
```

## Discord Application Setup

1. Go to the [Discord Developer Portal](https://discord.com/developers/applications)
2. Create a new application or use existing application with Client ID `1466997290819649619`
3. Navigate to the "OAuth2" section
4. Add the following redirect URI:
   - `http://localhost:8081/auth/discord/callback` (this points to the frontend)
5. Note down your Client ID and Client Secret
6. Under "OAuth2 Scopes", ensure you have at least:
   - `identify` (to get user information)

## Database Setup

1. Create a PostgreSQL database:
```sql
CREATE DATABASE discord_org_hub;
```

2. Run database migrations:
```bash
cd backend
cargo install sqlx-cli
sqlx migrate run
```

## Running the Application

1. Start the backend server:
```bash
cd backend
cargo run
```
The backend will run on `http://localhost:8080`

2. In a new terminal, start the frontend:
```bash
cd frontend
dx serve --port 8081
```
The frontend will run on `http://localhost:8081`

3. Open your browser and navigate to `http://localhost:8081`

**Important**: The Discord OAuth flow works as follows:
- Frontend (8081) → Discord OAuth → Frontend (8081) → Backend (8080) → Frontend (8081)
- The Discord redirect URI points to the frontend (8081) which handles the callback
- The frontend then calls the backend (8080) to exchange the authorization code for tokens
- After successful authentication, the backend redirects back to the frontend (8081)
- The `FRONTEND_URL` environment variable controls where users are redirected after OAuth

## Discord OAuth Flow

The authentication flow works as follows:

1. User clicks "Sign in with Discord" on the home page
2. User is redirected to Discord OAuth authorization page
3. After authorization, Discord redirects back to `/auth/discord/callback`
4. Backend exchanges the authorization code for an access token
5. Backend fetches user information from Discord API (`/users/@me`)
6. User information is stored in the database
7. Access token is stored for future API calls
8. User is redirected back to home page with authentication status

## API Endpoints

### Authentication
- `GET /auth/discord/callback` - Discord OAuth callback handler

### Users
- `GET /api/users` - List all users
- `POST /api/users` - Create a new user
- `GET /api/users/:id` - Get user by ID
- `PUT /api/users/:id` - Update user
- `DELETE /api/users/:id` - Delete user
- `GET /api/users/discord/:discord_id` - Get user by Discord ID
- `GET /api/users/stats` - Get user statistics

### Discord Tokens
- `POST /api/discord-tokens` - Create/update Discord token
- `GET /api/discord-tokens/user/:user_id` - Get token by user ID
- `PUT /api/discord-tokens/user/:user_id` - Update token
- `DELETE /api/discord-tokens/user/:user_id` - Delete token
- `GET /api/discord-tokens/verify/:user_id` - Verify token validity
- `POST /api/discord-tokens/cleanup` - Clean up expired tokens

### Health
- `GET /health` - API health check
- `GET /health/db` - Database health check

## Troubleshooting

### Common Issues

1. **Database connection fails**
   - Ensure PostgreSQL is running
   - Check DATABASE_URL format
   - Verify database exists and user has proper permissions

2. **Discord OAuth fails**
   - Verify DISCORD_CLIENT_ID and DISCORD_CLIENT_SECRET are correct
   - Check that redirect URI matches exactly in Discord application settings
   - Ensure Discord application has proper OAuth scopes configured

3. **CORS errors**
   - Backend includes CORS headers for localhost:8080
   - If running on different ports, update CORS configuration in backend

4. **Frontend build fails**
   - Ensure Dioxus CLI is installed: `cargo install dioxus-cli`
   - Check that all dependencies are properly specified

### Development Tips

- Use `cargo watch -x run` in the backend directory for auto-reloading
- Use `dx serve --hot-reload` in the frontend directory for hot reloading
- Check browser developer tools for network requests and console errors
- Monitor backend logs for detailed error information

## Testing the Implementation

### Manual Testing Steps

1. Start the backend server (`cargo run` in `/backend`)
2. Start the frontend server (`dx serve --port 8081` in `/frontend`)
3. Navigate to `http://localhost:8081` in your browser
4. Click "Sign in with Discord"
5. Authorize the Discord application
6. Verify you're redirected back with success status
7. Check that user information is stored in the database

### Backend API Testing

Test the health endpoints:
```bash
curl http://localhost:8080/health
curl http://localhost:8080/health/db
```

Test user endpoints (after authentication):
```bash
# Get user by ID
curl http://localhost:8080/api/users/{user-id}

# Get user by Discord ID
curl http://localhost:8080/api/users/discord/{discord-id}
```

Note: The backend API runs on port 8080, while the frontend UI runs on port 8081.

### Database Verification

Check if user data is properly stored:
```sql
-- Connect to your database
psql postgresql://username:password@localhost/discord_org_hub

-- Check users table
SELECT * FROM users;

-- Check discord_tokens table
SELECT user_id, token_type, scope, expires_at FROM discord_tokens;
```

## Security Considerations

- Never commit `.env` files containing secrets
- Use HTTPS in production environments
- Regularly rotate Discord client secrets
- Implement proper session management for production use
- Consider implementing CSRF protection for production
- Store access tokens securely and implement proper token refresh logic
- Consider implementing rate limiting for API endpoints
- Validate all user inputs and sanitize data before database operations