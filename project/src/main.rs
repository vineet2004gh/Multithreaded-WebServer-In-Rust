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

    let home = warp::path::end().and(warp::fs::file("./home.html"));

    let login_page = warp::path("login_page.html").and(warp::fs::file("./login_page.html"));

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

    let routes = home
        .or(login_page)
        .or(user_route)
        .or(login_route)
        .or(private_route)
        .or(admin_only_route)
        .with(warp::cors().allow_any_origin())
        .recover(errors::handle_rejection);

    info!("Server starting with thread pool of 4 workers");
    warp::serve(routes)
        .run(([127, 0, 0, 1], 8447))
        .await;
}

fn with_users_db(
    users_db: UsersDb,
) -> impl Filter<Extract = (UsersDb,), Error = Infallible> + Clone {
    warp::any().map(move || users_db.clone())
}
