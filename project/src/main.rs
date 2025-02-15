// Run using: $env:RUST_LOG="info"; cargo run --release
// check on site 
use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::{Body, Client, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use hyper_tls::HttpsConnector;
use project::auth;
use log::{info, error};
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[derive(Debug)]
enum AppError {
    Hyper(hyper::Error),
    Http(hyper::http::Error),
    Io(std::io::Error),
}

impl From<hyper::Error> for AppError {
    fn from(err: hyper::Error) -> Self {
        AppError::Hyper(err)
    }
}

impl From<hyper::http::Error> for AppError {
    fn from(err: hyper::http::Error) -> Self {
        AppError::Http(err)
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err)
    }
}

async fn serve_file(file_path: &str) -> Result<Response<Body>, AppError> {
    let path = Path::new(file_path);
    if path.exists() {
        let mut file = File::open(path).await?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).await?;
        let body = Body::from(contents);
        Ok(Response::new(body))
    } else {
        Ok(Response::builder()
            .status(404)
            .body(Body::from("File not found"))
            .unwrap())
    }
}

async fn handle_request(
    req: Request<Body>,
) -> Result<Response<Body>, AppError> {
    let path = req.uri().path();
    let method = req.method();

    match (method, path) {
        (&hyper::Method::GET, "/login.html") => serve_file("login.html").await,
        (&hyper::Method::GET, "/register.html") => serve_file("register.html").await,
        (&hyper::Method::GET, "/hello.html") => serve_file("hello.html").await,
        (&hyper::Method::POST, "/register") => {
            let body_bytes = hyper::body::to_bytes(req.into_body()).await?;
            let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap_or_default();
            let username = body["username"].as_str().unwrap_or("");
            let password = body["password"].as_str().unwrap_or("");
            
            match auth::register_user(username, password) {
                Ok(_) => Ok(Response::builder()
                    .status(200)
                    .body(Body::from(serde_json::json!({"message": "Registration Successful"}).to_string()))
                    .unwrap()),
                Err(e) => Ok(Response::builder()
                    .status(400)
                    .body(Body::from(serde_json::json!({"error": e}).to_string()))
                    .unwrap()),
            }
        },
        (&hyper::Method::POST, "/login") => {
            let body_bytes = hyper::body::to_bytes(req.into_body()).await?;
            let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap_or_default();
            let username = body["username"].as_str().unwrap_or("");
            let password = body["password"].as_str().unwrap_or("");
            
            match auth::login_user(username, password) {
                Ok(token) => Ok(Response::builder()
                    .status(200)
                    .body(Body::from(serde_json::json!({"token": token}).to_string()))
                    .unwrap()),
                Err(e) => Ok(Response::builder()
                    .status(401)
                    .body(Body::from(serde_json::json!({"error": e}).to_string()))
                    .unwrap()),
            }
        },
        (&hyper::Method::GET, "/protected") => {
            if let Some(auth_header) = req.headers().get(hyper::header::AUTHORIZATION) {
                if let Ok(auth_str) = auth_header.to_str() {
                    if let Some(token) = auth_str.strip_prefix("Bearer ") {
                        match auth::verify_token(token) {
                            Ok(user) => {
                                info!("🔓 Access granted to user: {}", user);
                                Ok(Response::builder()
                                    .status(200)
                                    .body(Body::from(serde_json::json!({"user": user}).to_string()))
                                    .unwrap())
                            },
                            Err(_) => Ok(Response::builder()
                                .status(403)
                                .body(Body::from(serde_json::json!({"error": "Access Denied"}).to_string()))
                                .unwrap()),
                        }
                    } else {
                        Ok(Response::builder()
                            .status(401)
                            .body(Body::from(serde_json::json!({"error": "Invalid Token"}).to_string()))
                            .unwrap())
                    }
                } else {
                    Ok(Response::builder()
                        .status(401)
                        .body(Body::from(serde_json::json!({"error": "Invalid Authorization Header"}).to_string()))
                        .unwrap())
                }
            } else {
                Ok(Response::builder()
                    .status(401)
                    .body(Body::from(serde_json::json!({"error": "Missing Token"}).to_string()))
                    .unwrap())
            }
        },
        _ => serve_file("404.html").await
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);

    let make_svc = make_service_fn(move |_conn| {
        let client = client.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                async move {
                    match handle_request(req).await {
                        Ok(response) => Ok::<Response<Body>, Infallible>(response),
                        Err(e) => {
                            error!("Error handling request: {:?}", e);
                            Ok(Response::builder()
                                .status(500)
                                .body(Body::from("Internal Server Error"))
                                .unwrap())
                        }
                    }
                }
            }))
        }
    });

    let addr = SocketAddr::from(([127, 0, 0, 1], 8402));

    let server = Server::bind(&addr).serve(make_svc);

    info!("🚀 Reverse proxy server running on http://{}", addr);

    if let Err(e) = server.await {
        error!("server error: {}", e);
    }
}

