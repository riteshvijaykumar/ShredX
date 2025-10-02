use std::sync::Arc;
use warp::{Filter, http::StatusCode};
use sqlx::{PgPool, Row};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use jsonwebtoken::{encode, Header, EncodingKey};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: Uuid,
    username: String,
    email: String,
    role: String,
    created_at: chrono::DateTime<chrono::Utc>,
    last_login: Option<chrono::DateTime<chrono::Utc>>,
    is_active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginResponse {
    token: String,
    user: User,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    role: String,
}

pub fn create_auth_routes(pool: Arc<PgPool>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let login = warp::path("api")
        .and(warp::path("login"))
        .and(warp::post())
        .and(warp::body::json())
        .and(with_pool(pool.clone()))
        .and_then(login_handler);
    
    let logout = warp::path("api")
        .and(warp::path("logout"))
        .and(warp::post())
        .and_then(logout_handler);
    
    login.or(logout)
}

fn with_pool(pool: Arc<PgPool>) -> impl Filter<Extract = (Arc<PgPool>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || pool.clone())
}

fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    format!("{:x}", hasher.finalize())
}

async fn login_handler(
    body: LoginRequest,
    pool: Arc<PgPool>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let password_hash = hash_password(&body.password);
    
    let user_row = sqlx::query(
        "SELECT id, username, email, role, created_at, last_login, is_active 
         FROM users WHERE username = $1 AND password_hash = $2 AND is_active = true"
    )
    .bind(&body.username)
    .bind(&password_hash)
    .fetch_optional(pool.as_ref())
    .await;

    match user_row {
        Ok(Some(row)) => {
            let user = User {
                id: row.get("id"),
                username: row.get("username"),
                email: row.get("email"),
                role: row.get("role"),
                created_at: row.get("created_at"),
                last_login: row.get("last_login"),
                is_active: row.get("is_active"),
            };

            // Update last login
            let _ = sqlx::query("UPDATE users SET last_login = NOW() WHERE id = $1")
                .bind(user.id)
                .execute(pool.as_ref())
                .await;

            // Create JWT token
            let claims = Claims {
                sub: user.username.clone(),
                exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
                role: user.role.clone(),
            };

            let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key".to_string());
            let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
                .map_err(|_| warp::reject::custom(AuthError::TokenCreationError))?;

            let response = LoginResponse { token, user };
            
            Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({
                    "success": true,
                    "data": response,
                    "message": "Login successful"
                })),
                StatusCode::OK
            ))
        }
        Ok(None) => {
            Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({
                    "success": false,
                    "message": "Invalid credentials"
                })),
                StatusCode::UNAUTHORIZED
            ))
        }
        Err(e) => {
            tracing::error!("Database error during login: {}", e);
            Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({
                    "success": false,
                    "message": "Server error"
                })),
                StatusCode::INTERNAL_SERVER_ERROR
            ))
        }
    }
}

async fn logout_handler() -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::reply::json(&serde_json::json!({
        "success": true,
        "message": "Logged out successfully"
    })))
}

#[derive(Debug)]
struct AuthError {
    message: String,
}

impl AuthError {
    const TOKEN_CREATION_ERROR: AuthError = AuthError {
        message: String::new(),
    };
}

impl warp::reject::Reject for AuthError {}