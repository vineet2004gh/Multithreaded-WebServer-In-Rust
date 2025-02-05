use log::info;
use std::{collections::HashMap, convert::Infallible, sync::Arc};
use tokio::sync::Mutex;
use warp::{Filter, Rejection};

mod errors;
mod handlers;
mod models;
mod security;
mod threadpool;
use threadpool::ThreadPool;


type UsersDb = Arc<Mutex<HashMap<String, models::User>>>;
type Result<T> = std::result::Result<T, Rejection>;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    log4rs::init_file("logconfig.yml", Default::default()).expect("Log config file not found.");
    info!("Starting server...");
    let pool = Arc::new(ThreadPool::new(4));
    let users_db: UsersDb = Arc::new(Mutex::new(HashMap::new()));
    // In main.rs, replace the root route with:
    let root = warp::path::end().map(|| {
    warp::reply::html(r#"
        <html>
            <head>
                <title>Welcome to JWT Auth</title>
                <style>
                    .container { 
                        max-width: 400px; 
                        margin: 50px auto; 
                        padding: 20px;
                        border: 1px solid #ccc;
                        border-radius: 5px;
                    }
                    .form-group { margin-bottom: 15px; }
                    input, select { width: 100%; padding: 8px; margin-top: 5px; }
                    button { width: 100%; padding: 10px; background: #007bff; color: white; border: none; }
                </style>
            </head>
            <body>
                <div class="container">
                    <h1>Welcome</h1>
                    <form id="loginForm">
                        <div class="form-group">
                            <label for="username">Username:</label>
                            <input type="text" id="username" name="username" required>
                        </div>
                        <div class="form-group">
                            <label for="password">Password:</label>
                            <input type="password" id="password" name="password" required>
                        </div>
                        <div class="form-group">
                            <label for="role">Role:</label>
                            <select id="role" name="role">
                                <option value="user">User</option>
                                <option value="admin">Admin</option>
                            </select>
                        </div>
                        <div class="form-group">
                            <button type="button" onclick="submitForm()">Login</button>
                            <button type="button" onclick="register()">Register</button>
                        </div>
                    </form>
                </div>
                <script>
                    async function submitForm() {
                        const username = document.getElementById('username').value;
                        const password = document.getElementById('password').value;
                        
                        if (!username || !password) {
                            alert('Please fill in all fields');
                            return;
                        }

                        try {
                            const response = await fetch('/login', {
                                method: 'POST',
                                headers: {
                                    'Content-Type': 'application/json',
                                    'Accept': 'application/json'
                                },
                                body: JSON.stringify({
                                    username: username,
                                    password: password
                                })
                            });
                            
                            if (response.ok) {
                                const data = await response.text();
                                localStorage.setItem('jwt_token', data);
                                // Add authorization header for subsequent requests
                                const privateResponse = await fetch('/private', {
                                    headers: {
                                        'Authorization': `Bearer ${data}`
                                    }
                                });
                                
                                if (privateResponse.ok) {
                                    window.location.href = '/private';
                                } else {
                                    throw new Error('Failed to access private page');
                                }
                            } else {
                                const errorData = await response.text();
                                alert(`Login failed: ${errorData}`);
                            }
                        } catch (error) {
                            console.error('Error:', error);
                            alert(`Login failed: ${error.message}`);
                        }
                    }

                    async function register() {
                        const username = document.getElementById('username').value;
                        const password = document.getElementById('password').value;
                        const role = document.getElementById('role').value;
                        
                        try {
                            const response = await fetch('/user', {
                                method: 'POST',
                                headers: {
                                    'Content-Type': 'application/json',
                                },
                                body: JSON.stringify({ username, password, role })
                            });
                            
                            if (response.ok) {
                                alert('Registration successful! Please login.');
                            } else {
                                alert('Registration failed');
                            }
                        } catch (error) {
                            console.error('Error:', error);
                            alert('Registration failed');
                        }
                    }
                </script>
            </body>
        </html>
    "#)
});

    let user_route = warp::path("user")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_users_db(users_db.clone()))
        .and_then(handlers::create_user);

    let login_route = warp::path("login")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_users_db(users_db.clone()))
        .and_then(handlers::login);

    let private_route = warp::path("private")
        .and(warp::get())
        .and(security::with_auth(security::Role::User))
        .and_then(handlers::get_private);

    let admin_only_route = warp::path("admin_only")
        .and(warp::get())
        .and(with_users_db(users_db))
        .and(security::with_auth(security::Role::Admin))
        .and_then(handlers::get_admin_only);

    let routes = root
        .or(user_route)
        .or(login_route)
        .or(private_route)
        .or(admin_only_route)
        .with(warp::cors().allow_any_origin())
        .recover(errors::handle_rejection);
    let pool_filter = pool.clone();
    let with_pool = warp::any().map(move || pool_filter.clone());
    let threaded_routes = routes.and(with_pool).map(move |response, pool: Arc<ThreadPool>| {
        let pool = pool.clone();
        pool.execute(move || {
            info!("Request being handled by worker thread");
        });
        response
    });
    info!("Server starting with thread pool of 4 workers");
    warp::serve(threaded_routes).run(([127, 0, 0, 1], 8447)).await;
}

fn with_users_db(
    users_db: UsersDb,
) -> impl Filter<Extract = (UsersDb,), Error = Infallible> + Clone {
    warp::any().map(move || users_db.clone())
}
