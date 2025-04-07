use crate::{db, errors, models, security, Result};
use cookie::{Cookie, SameSite};
use log::{error, info};
use std::{fs, path::Path};
use warp::{
    http::{Response, StatusCode},
    reject, Reply,
};

pub async fn create_user(user: models::CreateUser, db_pool: db::DbPool) -> Result<impl Reply> {
    info!("Create user, received UserData: {:?}", user);

    // Get a connection from the pool
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(reject::custom(errors::CustomError::InternalError));
        }
    };

    // Check if user exists
    if let Some(_) = db::UserRepository::find_by_username(&mut conn, &user.username) {
        error!("User already exists");
        return Err(reject::custom(errors::CustomError::UserExistsError(
            user.username,
        )));
    }

    // Hash the password
    let hashed_password = security::get_hashed_password(&user.password);

    // Create new user object
    let new_user = models::NewUser {
        username: user.username.clone(),
        password: hashed_password,
        role: user.role.clone(),
    };

    // Insert user into database
    match db::UserRepository::create_user(&mut conn, &new_user) {
        Ok(created_user) => {
            info!("User {} added.", &user.username);
            Ok(Response::builder()
                .status(StatusCode::CREATED)
                .body(serde_json::to_string(&created_user).unwrap()))
        }
        Err(e) => {
            error!("Failed to create user: {}", e);
            Err(reject::custom(errors::CustomError::InternalError))
        }
    }
}

pub async fn login(login_user: models::LoginUser, db_pool: db::DbPool) -> Result<impl Reply> {
    info!("Received login request...");

    // Get a connection from the pool
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(reject::custom(errors::CustomError::InternalError));
        }
    };

    // Find user in database
    let user = match db::UserRepository::find_by_username(&mut conn, &login_user.username) {
        Some(user) => user,
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
    let token = security::get_jwt_for_user(&user);

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

pub async fn get_admin_only(db_pool: db::DbPool, username: String) -> Result<impl Reply> {
    info!("Return admin only page.");

    // Get a connection from the pool
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(reject::custom(errors::CustomError::InternalError));
        }
    };

    // Count users in database
    let user_count = db::UserRepository::count_users(&mut conn);

    let template_path =
        Path::new("e:/Rust/Multithreaded-WebServer-In-Rust/project/admin_only.html");

    match fs::read_to_string(template_path) {
        Ok(template) => {
            // Replace the placeholder with the username and user count
            let html =
                template
                    .replacen("{}", &username, 1)
                    .replacen("{}", &user_count.to_string(), 1);

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
