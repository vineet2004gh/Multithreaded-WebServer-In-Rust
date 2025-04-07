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
