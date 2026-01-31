use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Headers, Request, RequestInit, Response};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub discord_id: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
}

pub struct ApiService;

impl ApiService {
    const BASE_URL: &'static str = "http://localhost:8080/api";

    /// Make a GET request to the API
    pub async fn get(endpoint: &str) -> Result<String, JsValue> {
        let url = format!("{}{}", Self::BASE_URL, endpoint);

        let opts = RequestInit::new();
        opts.set_method("GET");

        let headers = Headers::new()?;
        headers.set("Content-Type", "application/json")?;
        opts.set_headers(&headers);

        let request = Request::new_with_str_and_init(&url, &opts)?;

        let window = web_sys::window().ok_or("No window object")?;
        let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

        let resp: Response = resp_value.dyn_into()?;
        let text = JsFuture::from(resp.text()?).await?;

        Ok(text.as_string().unwrap_or_default())
    }

    /// Make a POST request to the API
    pub async fn post(endpoint: &str, body: &str) -> Result<String, JsValue> {
        let url = format!("{}{}", Self::BASE_URL, endpoint);

        let mut opts = RequestInit::new();
        opts.set_method("POST");
        opts.set_body(&JsValue::from_str(body));

        let headers = Headers::new()?;
        headers.set("Content-Type", "application/json")?;
        opts.set_headers(&headers);

        let request = Request::new_with_str_and_init(&url, &opts)?;

        let window = web_sys::window().ok_or("No window object")?;
        let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

        let resp: Response = resp_value.dyn_into()?;
        let text = JsFuture::from(resp.text()?).await?;

        Ok(text.as_string().unwrap_or_default())
    }

    /// Get user by Discord ID
    pub async fn get_user_by_discord_id(discord_id: &str) -> Result<User, String> {
        let endpoint = format!("/users/discord/{}", discord_id);

        match Self::get(&endpoint).await {
            Ok(response_text) => serde_json::from_str::<User>(&response_text)
                .map_err(|e| format!("Failed to parse user data: {}", e)),
            Err(e) => Err(format!("Failed to fetch user: {:?}", e)),
        }
    }

    /// Get user by user ID
    pub async fn get_user_by_id(user_id: &str) -> Result<User, String> {
        let endpoint = format!("/users/{}", user_id);

        match Self::get(&endpoint).await {
            Ok(response_text) => serde_json::from_str::<User>(&response_text)
                .map_err(|e| format!("Failed to parse user data: {}", e)),
            Err(e) => Err(format!("Failed to fetch user: {:?}", e)),
        }
    }

    /// Create a new user
    pub async fn create_user(user_data: CreateUserRequest) -> Result<User, String> {
        let body = serde_json::to_string(&user_data)
            .map_err(|e| format!("Failed to serialize user data: {}", e))?;

        match Self::post("/users", &body).await {
            Ok(response_text) => serde_json::from_str::<User>(&response_text)
                .map_err(|e| format!("Failed to parse user data: {}", e)),
            Err(e) => Err(format!("Failed to create user: {:?}", e)),
        }
    }

    /// Check health of the API
    pub async fn health_check() -> Result<HealthResponse, String> {
        match Self::get("/health").await {
            Ok(response_text) => serde_json::from_str::<HealthResponse>(&response_text)
                .map_err(|e| format!("Failed to parse health response: {}", e)),
            Err(_) => Err("Failed to reach API".to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub discord_id: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
}
