use std::sync::Arc;
use warp::Filter;
use sqlx::PgPool;

mod auth;
mod drives;
mod sanitization;

pub fn create_api_routes(pool: Arc<PgPool>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let api_base = warp::path("api");
    
    let health = api_base
        .and(warp::path("health"))
        .and(warp::get())
        .and_then(health_handler);
    
    let auth_routes = auth::create_auth_routes(pool.clone());
    let drive_routes = drives::create_drive_routes(pool.clone());
    let sanitization_routes = sanitization::create_sanitization_routes(pool.clone());
    
    health
        .or(auth_routes)
        .or(drive_routes)
        .or(sanitization_routes)
}

async fn health_handler() -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::reply::json(&serde_json::json!({
        "status": "healthy",
        "service": "HDD Tool Server",
        "timestamp": chrono::Utc::now()
    })))
}