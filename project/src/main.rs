use log::info;
use std::{collections::HashMap, convert::Infallible, fs, sync::Arc};
use tokio::sync::Mutex;
use warp::{Filter, Rejection};

mod errors;
mod handlers;
mod models;
mod proxy_server;
mod security;
mod template_handler; // Add the template_handler module
mod threadpool;
type UsersDb = Arc<Mutex<HashMap<String, models::User>>>;
type Result<T> = std::result::Result<T, Rejection>;

#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() {
    dotenv::dotenv().ok();
    log4rs::init_file("logconfig.yml", Default::default()).expect("Log config file not found.");
    info!("Starting server...");

    // Start the proxy server in a separate task
    tokio::spawn(proxy_server::start_proxy_server());

    // Load server config with more resilient path handling
    let config_content = match fs::read_to_string(
        "E:/Rust/Multithreaded-WebServer-In-Rust/project/src/config/server_config.toml",
    ) {
        Ok(content) => content,
        Err(_) => {
            // Try with project-relative path as fallback
            fs::read_to_string("../server_config.toml")
                .expect("Failed to read server configuration file")
        }
    };

    let config: toml::Value =
        toml::from_str(&config_content).expect("Failed to parse server configuration");

    // Setup load balancer if enabled
    let load_balancer = if config["load_balancing"]["enabled"]
        .as_bool()
        .unwrap_or(false)
    {
        let servers = config["load_balancing"]["upstream_servers"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect::<Vec<_>>();

        Some(Arc::new(template_handler::RoundRobinBalancer::new(servers)))
    } else {
        None
    };

    let num_threads = 4;
    let base_port = 8447;
    let users_db: UsersDb = Arc::new(Mutex::new(HashMap::new()));

    let mut join_handles = Vec::new();

    // Start individual web servers on different ports
    for thread_id in 0..num_threads {
        let port = base_port + thread_id;
        let users_db = users_db.clone();
        let load_balancer = load_balancer.clone();

        // Create a new route for each port
        let join_handle = tokio::spawn(async move {
            // Load the HTML file and replace placeholders
            let root = warp::path::end().map(move || {
                let server_address = format!("127.0.0.1:{}", port);

                // Create template values
                let mut template_values = template_handler::create_template_values(
                    port,
                    &thread_id.to_string(),
                    &server_address,
                );

                // Get upstream server if load balancing is enabled
                let upstream_server = if let Some(lb) = &load_balancer {
                    Some(lb.get_next_server())
                } else {
                    None
                };

                // Process the template
                let html_content = template_handler::process_template(
                    "./login_page.html",
                    template_values,
                    load_balancer.is_some(),
                    upstream_server,
                );

                warp::reply::html(html_content)
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
                .and(with_users_db(users_db.clone()))
                .and(security::with_auth(security::Role::Admin))
                .and_then(handlers::get_admin_only);

            let routes = root
                .or(user_route)
                .or(login_route)
                .or(private_route)
                .or(admin_only_route)
                .with(warp::cors().allow_any_origin())
                .recover(errors::handle_rejection);

            info!("Thread {} starting server on port {}", thread_id, port);
            warp::serve(routes).run(([127, 0, 0, 1], port)).await;
        });

        join_handles.push(join_handle);
    }

    // Wait for all servers to complete (they won't in normal operation)
    for handle in join_handles {
        if let Err(e) = handle.await {
            eprintln!("Server thread error: {}", e);
        }
    }
}

fn with_users_db(
    users_db: UsersDb,
) -> impl Filter<Extract = (UsersDb,), Error = Infallible> + Clone {
    warp::any().map(move || users_db.clone())
}
