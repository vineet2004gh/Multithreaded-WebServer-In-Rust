use hyper::{Body, Client, Request, Uri};
use log::info;
use std::{
    fs,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use warp::{http, Filter, Rejection, Reply};

pub struct RoundRobinBalancer {
    servers: Vec<String>,
    next: AtomicUsize,
}

impl RoundRobinBalancer {
    pub fn new(servers: Vec<String>) -> Self {
        RoundRobinBalancer {
            servers,
            next: AtomicUsize::new(0),
        }
    }

    pub fn get_next_server(&self) -> String {
        let current = self.next.fetch_add(1, Ordering::SeqCst) % self.servers.len();
        self.servers[current].clone()
    }
}

pub async fn start_proxy_server() {
    info!("Starting proxy server...");

    // Load the proxy configuration
    let config_content = match fs::read_to_string("./server_config.toml") {
        Ok(content) => content,
        Err(_) => {
            // Try with alternative paths
            fs::read_to_string("./src/config/server_config.toml")
                .expect("Failed to read proxy configuration file")
        }
    };

    // Parse the TOML configuration
    let config: toml::Value =
        toml::from_str(&config_content).expect("Failed to parse proxy configuration");

    let mode = config["reverse_proxy"]["enabled"]
        .as_bool()
        .unwrap_or(false);

    if !mode {
        info!("Reverse proxy is disabled in configuration. Exiting proxy server.");
        return;
    }

    let proxy_port = config["server"]["port"].as_integer().unwrap_or(8080) as u16;

    // Setup load balancer
    let upstream_servers = config["load_balancing"]["upstream_servers"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect::<Vec<_>>();

    let balancer = Arc::new(RoundRobinBalancer::new(upstream_servers.clone()));

    info!(
        "Proxy server configured with {} upstream servers:",
        upstream_servers.len()
    );
    for (i, server) in upstream_servers.iter().enumerate() {
        info!("  Server {}: {}", i + 1, server);
    }

    // Set up the HTTP client for proxying requests
    let client = Client::new();
    let balancer_filter = warp::any().map(move || balancer.clone());
    let client_filter = warp::any().map(move || client.clone());

    // Create a route that will match any path and proxy the request
    let proxy_route = warp::path::full()
        .and(warp::method())
        .and(warp::header::headers_cloned())
        .and(warp::body::bytes())
        .and(balancer_filter)
        .and(client_filter)
        .and_then(handle_proxy_request);

    info!("Starting reverse proxy server on port {}", proxy_port);
    warp::serve(proxy_route)
        .run(([127, 0, 0, 1], proxy_port))
        .await;
}

async fn handle_proxy_request(
    path: warp::path::FullPath,
    method: http::Method,
    headers: http::HeaderMap,
    body: hyper::body::Bytes,
    balancer: Arc<RoundRobinBalancer>,
    client: hyper::Client<hyper::client::HttpConnector>,
) -> Result<impl Reply, Rejection> {
    // Get the next upstream server using the round-robin balancer
    let upstream_server = balancer.get_next_server();

    info!(
        "Proxying request to {} - {}{}",
        method,
        upstream_server,
        path.as_str()
    );

    // Build the URL to the upstream server
    let uri_string = format!("{}{}", upstream_server, path.as_str());
    let uri: Uri = uri_string.parse().map_err(|_| warp::reject::not_found())?;

    // Prepare the request to the upstream server
    let mut req_builder = Request::builder().method(method).uri(uri);

    // Copy the headers from the original request
    let req_headers = req_builder.headers_mut().unwrap();
    for (key, value) in headers.iter() {
        // Skip the host header as it will be set by the client
        if key != http::header::HOST {
            req_headers.insert(key, value.clone());
        }
    }

    // Add headers to indicate this is a proxied request
    req_headers.insert("X-Forwarded-By", "Rust-Proxy/1.0".parse().unwrap());
    req_headers.insert("X-Forwarded-Proto", "http".parse().unwrap());

    // Send the request to the upstream server
    let proxy_req = req_builder
        .body(Body::from(body))
        .map_err(|_| warp::reject::reject())?;

    let res = client
        .request(proxy_req)
        .await
        .map_err(|_| warp::reject::reject())?;

    // Build and return the response
    let (parts, body) = res.into_parts();
    let mut response_builder = http::Response::builder().status(parts.status);

    // Copy the response headers
    let resp_headers = response_builder.headers_mut().unwrap();
    for (key, value) in parts.headers.iter() {
        resp_headers.insert(key, value.clone());
    }

    // Add a header to indicate the upstream server used
    resp_headers.insert("X-Upstream-Server", upstream_server.parse().unwrap());

    let body_bytes = hyper::body::to_bytes(body)
        .await
        .map_err(|_| warp::reject::reject())?;

    // Create response and apply status
    let mut response = warp::reply::Response::new(body_bytes.into());
    *response.status_mut() = parts.status;

    // Add all headers from resp_headers
    for (key, value) in resp_headers.iter() {
        response.headers_mut().insert(key, value.clone());
    }

    Ok(response)
}
