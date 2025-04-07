# Multithreaded-WebServer-In-Rust

A high-performance, multithreaded web server built with Rust featuring user authentication, role-based access control, and load balancing capabilities.

## Features

- Multithreaded architecture using Tokio and threadpool
- User authentication with JWT tokens
- Role-based access control (User/Admin roles)
- Database integration with PostgreSQL using Diesel ORM
- Load balancing with round-robin algorithm
- Reverse proxy functionality using Hyper
- Template-based HTML rendering
- Comprehensive logging with log4rs
- Secure password hashing with scrypt
- Cookie-based session management

## Prerequisites

- Rust (latest stable version)
- PostgreSQL database server
- Diesel CLI (`cargo install diesel_cli --no-default-features --features postgres`)

## Setup Instructions

### 1. Database Setup

1. Install PostgreSQL and create a new database:

   ```bash
   createdb users_db
   ```

2. Update the database connection string in `.env` file:

   ```
   DATABASE_URL = "postgres://username:password@localhost:5432/users_db"
   ```

   Replace `username` and `password` with your PostgreSQL credentials.

3. Run the database migrations:
   ```bash
   cd project
   diesel migration run
   ```

### 2. Configuration

The server can be configured through multiple configuration files:

- `logconfig.yml` - Logging configuration
- `.env` - Environment variables and database configuration
- `diesel.toml` - Diesel ORM configuration

### 3. Running the Server

```bash
cd project
cargo run
```

The server will start multiple instances on different ports (default: 8447-8450) and the reverse proxy will be available on port 8080.

## Project Structure

- `src/main.rs` - Server initialization and threading logic
- `src/db.rs` - Database connection and operations using Diesel
- `src/handlers.rs` - HTTP request handlers using Warp
- `src/security.rs` - Authentication and authorization with JWT
- `src/errors.rs` - Custom error types and error handling
- `src/models.rs` - Data models and structures
- `src/proxy_server.rs` - Reverse proxy implementation using Hyper
- `src/template_handler.rs` - HTML template processing
- `src/schema.rs` - Database schema definitions
- `src/config/` - Configuration files

## Authentication Flow

1. Register a user at `/user` endpoint (POST)
2. Login at `/login` endpoint (POST)
3. Access protected routes:
   - `/private` - Available to all authenticated users
   - `/admin_only` - Available only to users with Admin role

## Dependencies

The project uses several key dependencies:

- `tokio` - Asynchronous runtime
- `warp` - Web framework
- `hyper` - HTTP client/server
- `diesel` - PostgreSQL ORM
- `jsonwebtoken` - JWT handling
- `log4rs` - Logging framework
- `scrypt` - Password hashing
- `serde` - Serialization/deserialization

## Troubleshooting

### Database Connection Issues

If you encounter database connection errors, verify:

1. PostgreSQL is running
2. Database credentials in `.env` file are correct
3. The database `users_db` exists
4. User has proper permissions

Error message: `FATAL: password authentication failed for user "postgres"` indicates incorrect password in your DATABASE_URL.

### Logging

Logs are configured in `logconfig.yml`. By default, logs are written to:

- Console output
- File logs in the project directory

### Common Issues

1. **Port Conflicts**: If you see port binding errors, ensure no other services are using ports 8080 or 8447-8450
2. **Database Migrations**: If migrations fail, try running `diesel migration redo`
3. **Environment Variables**: Ensure `.env` file is properly formatted and in the correct location
# 🦀RustyGate: The Multi-Threaded Web Server🚀

## 🔎Overview
This project is a high-performance **multi-threaded web server** built in the fundamental structure of Rust. It features authentication using JSONWebToken, reverse proxying, load balancing, and a simple PostgreSQL backend. The server efficiently handles multiple requests using a thread pool, ensuring scalability and responsiveness. Implementation on Rust offers high performance, memory safety, and fearless concurrency without needing a garbage collector and with its powerful async ecosystem and strict compile-time checks make it perfect for building a secure, scalable, and efficient multi-threaded web server preventing from possible DoS attacks. 

## 🌟Features
- **Multi-threading**: Uses a thread pool for efficient request handling.
- **JWT Authentication**: Secure user authentication with JSON Web Tokens.
- **Reverse Proxy & Load Balancing**: Distributes incoming requests to backend servers.
- **PostgreSQL Integration**: Stores user data securely.
- **Error Handling**: Manages HTTP status codes and application errors gracefully.
- **User & Admin Roles**:
  - Users access a public page.
  - Admins have access to a protected section.

## 📐Project Structure
The project is organized into several modules:
- `main.rs`: Initializes the server, sets up logging, starts the proxy server, and manages the thread pool.
- `db.rs`: Manages PostgreSQL database connections and queries using Diesel.
- `errors.rs`: Defines custom error types and handling logic.
- `handlers.rs`: Contains API request handlers for authentication and user management.
- `models.rs`: Defines data structures for users and authentication.
- `proxy_server.rs`: Implements the reverse proxy and load balancing.
- `schema.rs`: Defines database table mappings for Diesel ORM.
- `security.rs`: Handles JWT authentication and password hashing.
- `template_handler.rs`: Renders HTML templates for the website.
- `threadpool.rs`: Implements the custom thread pool for handling concurrent requests.

## 📜Dependencies
```toml
tokio = { version = "1", features = ["full"] }
warp = "0.3"
hyper = { version = "0.14", features = ["full", "http1", "http2", "client"] }
hyper-tls = "0.5"
log = "0.4"
log4rs = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.5"
jsonwebtoken = "8"
chrono = { version = "0.4", features = ["serde"] }
dotenv = "0.15"
rand = "0.8"
threadpool = "1.8"
scrypt = "0.11.0"
cookie = "0.18.1"
thiserror = "2.0.12"
diesel = { version = "2.1.0", features = ["postgres", "r2d2", "chrono"] }
r2d2 = "0.8.10"
```

## 🛠️🛠Installation & Setup
1. **Clone the repository:**
   ```bash
   git clone https://github.com/vineet2004gh/Multithreaded-WebServer-In-Rust.git
   cd Multithreaded-WebServer-In-Rust/project
   ```
2. **Set up environment variables:**
   - Create a `.env` file and specify database credentials and JWT secrets.
3. **Install dependencies:**
   ```sh
   cargo build --release
   ```
4. **Run the server:**
   ```sh
   cargo run --release
   ```

## 🧲Usage
- Open your browser and visit `http://localhost:8447`.
- Authenticate using JWT to access protected routes.
- Admins can log in and manage users via the admin panel.

## ⚡API Endpoints
- `/` - Public home page.
- `/login_page` - User authentication.
- `/admin_only` - Protected admin route (requires JWT token).
- `/private_page` - Fetch user details (protected).

## 🛡️Security Features
- **JWT Authentication**: Secure user sessions with token-based authentication.
- **Password Hashing**: Uses `scrypt` for secure password storage.
- **HTTPS Support**: `hyper-tls` ensures encrypted communication.

## 🔁Load Balancing & Reverse Proxy
- The server distributes incoming requests across multiple backend instances.
- `proxy_server.rs` manages load balancing to ensure optimal resource utilization.

## 🤝Contributing
Contributions and Raising Issues are welcome! Feel free to raise a pull-request!

## 📚Authors & Contributors
Developed by **Team RustyGate**:
- Aman Revankar
- Vineet
- Pal Patel
- Srishti
- Jeferson

