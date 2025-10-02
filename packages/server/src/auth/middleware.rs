use std::convert::Infallible;
use warp::{Filter, http::header::AUTHORIZATION};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub role: String,
}

#[derive(Debug)]
pub struct AuthError;

impl warp::reject::Reject for AuthError {}

pub fn with_auth() -> impl Filter<Extract = (Claims,), Error = warp::Rejection> + Clone {
    warp::header::optional::<String>(AUTHORIZATION.as_str())
        .and_then(authorize)
}

async fn authorize(token: Option<String>) -> Result<Claims, warp::Rejection> {
    match token {
        Some(token) => {
            let token = token.strip_prefix("Bearer ").unwrap_or(&token);
            let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key".to_string());
            
            match decode::<Claims>(
                token,
                &DecodingKey::from_secret(secret.as_ref()),
                &Validation::default(),
            ) {
                Ok(token_data) => Ok(token_data.claims),
                Err(_) => Err(warp::reject::custom(AuthError)),
            }
        }
        None => Err(warp::reject::custom(AuthError)),
    }
}