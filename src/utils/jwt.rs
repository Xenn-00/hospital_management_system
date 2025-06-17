use std::sync::Arc;

use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::error_handling::app_error::AppError;

#[derive(Clone)]
pub struct JwtKeys {
    pub encoding: Arc<EncodingKey>,
    pub decoding: Arc<DecodingKey>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: i32,
    pub role: String,
    pub username: String,
    pub iat: usize,
    pub exp: usize,
}

pub fn generate_jwt(claims: &Claims, keys: &JwtKeys) -> Result<String, AppError> {
    encode(&Header::new(Algorithm::RS256), claims, &keys.encoding).map_err(|e| {
        error!("JWT encode error: {:?}", e);
        AppError::Internal(format!("Token generation failed"))
    })
}

pub fn decode_jwt(token: &str, keys: &JwtKeys) -> Result<Claims, AppError> {
    let token_data = decode::<Claims>(token, &keys.decoding, &Validation::new(Algorithm::RS256))
        .map_err(|e| {
            tracing::warn!("JWT decode error: {:?}", e);
            AppError::AuthError(format!("Invalid or expired token"))
        })?;

    Ok(token_data.claims)
}
