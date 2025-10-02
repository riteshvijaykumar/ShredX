use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::Filter;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Pool, Postgres};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tracing::{info, warn, error};

mod auth;
mod database;
mod handlers;
mod models;
mod sanitization;

use crate::auth::Claims;
use crate::models::*;

#[derive(Debug, Clone)]
pub struct AppState {
    pub db: PgPool,
    pub jwt_secret: String,
    pub active_operations: Arc<RwLock<HashMap<Uuid, SanitizationStatus>>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting HDD Tool Server...");

    // Load environment variables
    dotenv::dotenv().ok();
    
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://hdd_user:root@localhost/hdd_tool_db".to_string());
    
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "your-secret-key".to_string());
    
    let server_port: u16 = std::env::var("SERVER_PORT")
        .unwrap_or_else(|_| "3030".to_string())
        .parse()
        .unwrap_or(3030);

    // Connect to database
    info!("Connecting to database...");
    let db = PgPool::connect(&database_url).await?;
    
    // Run migrations
    sqlx::migrate!("./migrations").run(&db).await?;
    info!("Database migrations completed");

    // Create app state
    let app_state = AppState {
        db,
        jwt_secret,
        active_operations: Arc::new(RwLock::new(HashMap::new())),
    };

    // CORS configuration
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type", "authorization"])
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"]);

    // API routes
    let api = warp::path("api")
        .and(warp::path("v1"))
        .and(
            auth_routes(app_state.clone())
                .or(user_routes(app_state.clone()))
                .or(device_routes(app_state.clone()))
                .or(sanitization_routes(app_state.clone()))
                .or(certificate_routes(app_state.clone()))
        )
        .with(cors);

    // Health check route
    let health = warp::path("health")
        .and(warp::get())
        .map(|| warp::reply::json(&serde_json::json!({"status": "ok"})));

    let routes = health.or(api);

    info!("Server starting on port {}", server_port);
    warp::serve(routes)
        .run(([0, 0, 0, 0], server_port))
        .await;

    Ok(())
}

fn auth_routes(
    app_state: AppState,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let register = warp::path("auth")
        .and(warp::path("register"))
        .and(warp::post())
        .and(warp::body::json())
        .and(with_state(app_state.clone()))
        .and_then(handlers::auth::register);

    let login = warp::path("auth")
        .and(warp::path("login"))
        .and(warp::post())
        .and(warp::body::json())
        .and(with_state(app_state.clone()))
        .and_then(handlers::auth::login);

    register.or(login)
}

fn user_routes(
    app_state: AppState,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let get_profile = warp::path("user")
        .and(warp::path("profile"))
        .and(warp::get())
        .and(with_auth(app_state.clone()))
        .and_then(handlers::user::get_profile);

    get_profile
}

fn device_routes(
    app_state: AppState,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let list_devices = warp::path("devices")
        .and(warp::get())
        .and(with_auth(app_state.clone()))
        .and_then(handlers::device::list_devices);

    let get_device_info = warp::path("devices")
        .and(warp::path::param::<String>())
        .and(warp::get())
        .and(with_auth(app_state.clone()))
        .and_then(handlers::device::get_device_info);

    list_devices.or(get_device_info)
}

fn sanitization_routes(
    app_state: AppState,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let start_sanitization = warp::path("sanitize")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_auth(app_state.clone()))
        .and_then(handlers::sanitization::start_sanitization);

    let get_status = warp::path("sanitize")
        .and(warp::path::param::<Uuid>())
        .and(warp::path("status"))
        .and(warp::get())
        .and(with_auth(app_state.clone()))
        .and_then(handlers::sanitization::get_status);

    let stop_sanitization = warp::path("sanitize")
        .and(warp::path::param::<Uuid>())
        .and(warp::path("stop"))
        .and(warp::post())
        .and(with_auth(app_state.clone()))
        .and_then(handlers::sanitization::stop_sanitization);

    start_sanitization.or(get_status).or(stop_sanitization)
}

fn certificate_routes(
    app_state: AppState,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let get_certificates = warp::path("certificates")
        .and(warp::get())
        .and(with_auth(app_state.clone()))
        .and_then(handlers::certificate::list_certificates);

    let get_certificate = warp::path("certificates")
        .and(warp::path::param::<Uuid>())
        .and(warp::get())
        .and(with_auth(app_state.clone()))
        .and_then(handlers::certificate::get_certificate);

    get_certificates.or(get_certificate)
}

fn with_state(
    app_state: AppState,
) -> impl Filter<Extract = (AppState,), Error = Infallible> + Clone {
    warp::any().map(move || app_state.clone())
}

fn with_auth(
    app_state: AppState,
) -> impl Filter<Extract = (Claims, AppState), Error = warp::Rejection> + Clone {
    warp::header::<String>("authorization")
        .and(with_state(app_state))
        .and_then(|token: String, state: AppState| async move {
            let token = token.strip_prefix("Bearer ").unwrap_or(&token);
            match auth::verify_token(token, &state.jwt_secret) {
                Ok(claims) => Ok((claims, state)),
                Err(_) => Err(warp::reject::custom(AuthError)),
            }
        })
}

#[derive(Debug)]
struct AuthError;
impl warp::reject::Reject for AuthError {}