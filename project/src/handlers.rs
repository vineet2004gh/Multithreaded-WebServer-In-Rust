use crate::{errors, models, security, Result, UsersDb};
use log::{error, info};
use cookie::{Cookie,SameSite};
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
// pub async fn login(login_user: models::LoginUser, users_db: UsersDb) -> Result<impl Reply> {
//     info!("Received login request...");
//     let cur_user_db = users_db.lock().await;
//     let user = match cur_user_db.get(&login_user.username) {
//         Some(k) => k,
//         None => {
//             error!("User '{}' not found in database", &login_user.username);
//             return Err(reject::custom(errors::CustomError::InvalidCredentialsError));
//         }
//     };

//     info!("User found, verifying password...");
//     if !security::verify_password(&login_user.password, &user.password) {
//         error!("Password incorrect for user: {}", &login_user.username);
//         return Err(reject::custom(errors::CustomError::InvalidCredentialsError));
//     }

//     info!("Login success!");
//     let token = security::get_jwt_for_user(user);
//     Ok(Response::builder().status(StatusCode::OK).body(token))
// }

// In handlers.rs, modify the get_private function
pub async fn get_private(username: String) -> Result<impl Reply> {
    info!("Return private page.");

    Ok(warp::reply::html(format!(
        r#"
        <html>
            <head>
                <title>Private space</title>
                <style>
                    .container {{ 
                        max-width: 800px; 
                        margin: 50px auto; 
                        padding: 20px;
                    }}
                </style>
            </head>
            <body>
                <div class="container">
                    <h1>Private Area</h1>
                    <div>Welcome, {}</div>
                    <div>
                        <button onclick="logout()">Logout</button>
                        <button onclick="checkAdmin()">Admin Area</button>
                    </div>
                </div>
                <script>
                    function logout() {{
                        localStorage.removeItem('jwt_token');
                        window.location.href = '/';
                    }}

                    async function checkAdmin() {{
                        try {{
                            const response = await fetch('/admin_only', {{
                                headers: {{
                                    'Authorization': 'Bearer ' + localStorage.getItem('jwt_token')
                                }}
                            }});
                            
                            if (response.ok) {{
                                const html = await response.text();
                                document.body.innerHTML = html;
                            }} else {{
                                alert('Access denied');
                            }}
                        }} catch (error) {{
                            console.error('Error:', error);
                            alert('Access denied');
                        }}
                    }}
                </script>
            </body>
        </html>
        "#,
        username
    )))
}

pub async fn get_admin_only(users_db: UsersDb, username: String) -> Result<impl Reply> {
    info!("Return admin only page.");

    Ok(warp::reply::html(format!(
        r#"
    <html>
        <head>
            <title>Public space</title>
        <head>
        <body>
            <h1>Admin only</h1>
            <div>Logged in user: {}</div>
            <div>
                <b>Number of users in the database: {}</b>
            </div>
        </body>
    </html>
    "#,
        &username,
        users_db.lock().await.len()
    )))
}
