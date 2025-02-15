use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use serde::{Serialize, Deserialize};
use chrono::{Utc, Duration};
use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;

const SECRET_KEY: &[u8] = b"i3tmul71th73491n9";

// Global user database with thread safety
static USERS_DB: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

pub fn register_user(username: &str, password: &str) -> Result<(), String> {
    let hashed_password = hash(password, DEFAULT_COST).map_err(|_| "Password hashing failed")?;
    let mut db = USERS_DB.lock().map_err(|_| "Database lock failed")?;
    if db.contains_key(username) {
        return Err("Username already exists".to_string());
    }
    db.insert(username.to_string(), hashed_password);
    Ok(())
}

pub fn login_user(username: &str, password: &str) -> Result<String, String> {
    let db = USERS_DB.lock().map_err(|_| "Database lock failed")?;
    match db.get(username) {
        Some(hashed_password) => {
            if verify(password, hashed_password).map_err(|_| "Password verification failed")? {
                generate_token(username)
            } else {
                Err("Invalid username or password".to_string())
            }
        }
        None => Err("User not found".to_string()),
    }
}

pub fn generate_token(user_id: &str) -> Result<String, String> {
    let expiration = Utc::now() + Duration::hours(24);
    let claims = Claims {
        sub: user_id.to_owned(),
        exp: expiration.timestamp() as usize,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(SECRET_KEY))
        .map_err(|_| "Token generation failed".to_string())
}

pub fn verify_token(token: &str) -> Result<String, String> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(SECRET_KEY),
        &Validation::new(Algorithm::HS256),
    )
    .map(|data| data.claims.sub)
    .map_err(|_| "Invalid or Expired Token".to_string())
}

