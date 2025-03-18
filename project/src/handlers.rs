use crate::{errors, models, security, Result, UsersDb};
use cookie::{Cookie, SameSite};
use log::{error, info};
use std::fs;
use std::path::Path;
use warp::{
    http::{Response, StatusCode},
    reject, Reply,
};

pub async fn create_user(user: models::CreateUser, users_db: UsersDb) -> Result<impl Reply> {
    info!("Create user, received UserData: {:?}", user);
    let mut local_db = users_db.lock().await;

    if local_db.contains_key(&user.username) {
        error!("User already exists");
        return Err(reject::custom(errors::CustomError::UserExistsError(
            user.username,
        )));
    }

    info!("Adding user to the database...");
    let key_count = local_db.keys().len();
    let created_user = models::User {
        user_id: key_count,
        username: user.username,
        password: security::get_hashed_password(&user.password),
        role: user.role,
    };
    local_db.insert(created_user.username.clone(), created_user.clone());

    info!("User {} added.", &created_user.username);
    Ok(Response::builder()
        .status(StatusCode::CREATED)
        .body(serde_json::to_string(&created_user).unwrap()))
}

pub async fn login(login_user: models::LoginUser, users_db: UsersDb) -> Result<impl Reply> {
    info!("Received login request...");
    let cur_user_db = users_db.lock().await;
    let user = match cur_user_db.get(&login_user.username) {
        Some(k) => k,
        None => {
            error!("User '{}' not found in database", &login_user.username);
            return Err(reject::custom(errors::CustomError::InvalidCredentialsError));
        }
    };

    info!("User found, verifying password...");
    if !security::verify_password(&login_user.password, &user.password) {
        error!("Password incorrect for user: {}", &login_user.username);
        return Err(reject::custom(errors::CustomError::InvalidCredentialsError));
    }

    info!("Login success!");
    let token = security::get_jwt_for_user(user);

    // Create an HTTP‑only cookie with SameSite=Lax
    let mut jwt_cookie = Cookie::new("jwt", token);
    jwt_cookie.set_path("/");
    jwt_cookie.set_http_only(true);
    jwt_cookie.set_same_site(SameSite::Lax);

    let response = Response::builder()
        .status(StatusCode::OK)
        .header("set-cookie", jwt_cookie.to_string())
        .body("Login successful")
        .unwrap();

    Ok(response)
}

pub async fn get_private(username: String) -> Result<impl Reply> {
    info!("Return private page.");

    let template_path =
        Path::new("e:/Rust/Multithreaded-WebServer-In-Rust/project/private_page.html");

    match fs::read_to_string(template_path) {
        Ok(template) => {
            // Replace the placeholder with the username
            let html = template.replace("{}", &username);

            // Return successful response with the HTML content
            Ok(warp::reply::html(html))
        }
        Err(_) => {
            // Return error response if template file cannot be read
            Err(reject::custom(errors::CustomError::InternalError))
        }
    }
}

pub async fn get_admin_only(users_db: UsersDb, username: String) -> Result<impl Reply> {
    info!("Return admin only page.");

    let template_path =
        Path::new("e:/Rust/Multithreaded-WebServer-In-Rust/project/admin_only.html");

    match fs::read_to_string(template_path) {
        Ok(template) => {
            // Replace the placeholder with the username and user count
            let html = template.replacen("{}", &username, 1).replacen(
                "{}",
                &users_db.lock().await.len().to_string(),
                1,
            );

            // Return successful response with the HTML content
            Ok(warp::reply::html(html))
        }
        Err(_) => {
            // Return error response if template file cannot be read
            error!("Failed to read admin_only.html template");
            Err(reject::custom(errors::CustomError::InternalError))
        }
    }
}
