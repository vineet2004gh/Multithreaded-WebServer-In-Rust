# Multithreaded WebServer in Rust

A high-performance, multithreaded web server built with Rust featuring JWT-based authentication, role-based access control (RBAC), PostgreSQL integration with Diesel ORM, and reverse proxy load balancing.

## 🌟 Features

- **Multithreaded Architecture**: 4 worker threads running on ports 8447-8450, each handling HTTP requests independently
- **Reverse Proxy & Load Balancing**: Round-robin load balancer on port 8080 distributing requests across worker threads
- **JWT Authentication**: Secure token-based user authentication with configurable expiration (60 seconds)
- **Role-Based Access Control (RBAC)**: User and Admin roles with differentiated access levels
- **PostgreSQL Database**: User data persistence using Diesel ORM with R2D2 connection pooling
- **Secure Password Hashing**: Scrypt-based password hashing with random salt generation
- **Comprehensive Logging**: log4rs configured logging for debugging and monitoring
- **Cookie-Based Session Management**: HTTP-only, SameSite=Lax cookies for secure session handling
- **Custom Error Handling**: Application-specific error types with appropriate HTTP status codes

## 📋 Prerequisites

- Rust (latest stable version)
- PostgreSQL 12+
- Diesel CLI: `cargo install diesel_cli --no-default-features --features postgres`

## 🚀 Setup Instructions

### 1. Database Setup

Create PostgreSQL database:

```bash
createdb users_db
```

### 2. Environment Configuration

Copy `.env.example` to `.env` and update with your credentials:

```bash
cp project/.env.example project/.env
```

Update `project/.env`:

```env
JWT_SECRET="your-secret-key-here"
DATABASE_URL="postgres://postgres:password@localhost:5432/users_db"
```

### 3. Database Migrations

```bash
cd project
diesel migration run
```

This creates the `users` table with columns: `id`, `username`, `password`, `role`, `created_at`.

### 4. Build & Run

```bash
cd project
cargo build --release
cargo run --release
```

The server will start:
- Worker threads on ports **8447-8450**
- Reverse proxy on port **8080**

## 📁 Project Structure

```
src/
├── main.rs                 # Server initialization, thread spawning, route configuration
├── db.rs                   # PostgreSQL connection pool and UserRepository
├── handlers.rs             # HTTP handlers (login, register, private, admin routes)
├── security.rs             # JWT token generation, password hashing, RBAC logic
├── models.rs               # Data structures (User, Claims, CreateUser, LoginUser)
├── proxy_server.rs         # Reverse proxy and round-robin load balancer
├── template_handler.rs     # HTML template processing and placeholder replacement
├── errors.rs               # Custom error types and rejection handling
├── schema.rs               # Diesel ORM schema definitions
└── config/
    └── server_config.toml  # Server configuration (ports, load balancing, reverse proxy)

migrations/
└── 2025-03-18-192310_users_db/
    ├── up.sql              # Create users table
    └── down.sql            # Drop users table

HTML/
├── login_page.html         # Login/registration page (served on each worker)
├── private_page.html       # User-accessible protected page
└── admin_only.html         # Admin-only dashboard
```

## 🔐 Authentication & Authorization Flow

### Registration/Login Flow

1. **Register**: POST `/user`
   ```json
   {
     "username": "user123",
     "password": "securepass",
     "role": "user"
   }
   ```
   - Password is hashed using Scrypt with random salt
   - User stored in PostgreSQL

2. **Login**: POST `/login`
   ```json
   {
     "username": "user123",
     "password": "securepass"
   }
   ```
   - Password verified against stored hash using Scrypt
   - JWT token generated and set as HTTP-only cookie
   - Token includes username, role, and 60-second expiration

3. **Access Protected Routes**:
   - `/private` - GET (User or Admin)
   - `/admin_only` - GET (Admin only)
   - JWT validated from cookie on each request
   - Role-based authorization enforced (Admin role bypasses User checks)

## 🔄 Architecture Details

### Thread Model

- **4 Worker Threads**: Each listening on different port (8447-8450)
- **Proxy Thread**: Reverse proxy on port 8080 using Hyper client
- **Tokio Runtime**: Multi-threaded async runtime (4 worker threads) powering all HTTP servers
- **Connection Pooling**: R2D2 manages PostgreSQL connections across threads

### Load Balancing

- **Algorithm**: Round-robin via atomic counter increment per request
- **Balancer Location**: Reverse proxy distributes traffic to 4 worker instances
- **Upstream Servers**: Configured in `src/config/server_config.toml`
- **Request Routing**: All requests to port 8080 are balanced across 8447-8450

### Security Implementation

| Feature | Implementation |
|---------|----------------|
| **Password Hashing** | Scrypt with random salt (OsRng) |
| **Token Generation** | HMAC-SHA256 with 60-second expiration |
| **Session Storage** | HTTP-only, SameSite=Lax cookies |
| **Authorization** | Role comparison (Admin bypasses User checks) |
| **CORS** | Allow any origin (configurable) |

## 📝 API Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/` | None | Login page with server info |
| POST | `/user` | None | Register new user |
| POST | `/login` | None | Authenticate user |
| GET | `/private` | User+ | User dashboard |
| GET | `/admin_only` | Admin | Admin panel with user count |

## ⚙️ Configuration

### `src/config/server_config.toml`

```toml
[server]
threads = 4
port = 8080

[load_balancing]
enabled = true
type = "round_robin"
upstream_servers = [
    "http://127.0.0.1:8447",
    "http://127.0.0.1:8448",
    "http://127.0.0.1:8449",
    "http://127.0.0.1:8450",
]

[reverse_proxy]
enabled = true
```

### `logconfig.yml`

Configures log levels and output format. By default logs to console with timestamp and level.

## 🔧 Key Dependencies

- **tokio**: Async runtime with multi-threading support
- **warp**: Web framework built on Hyper
- **hyper**: HTTP client/server for proxy requests
- **diesel**: Type-safe ORM for PostgreSQL
- **jsonwebtoken**: JWT encoding/decoding
- **scrypt**: Password hashing algorithm
- **log4rs**: Structured logging framework
- **cookie**: HTTP cookie handling with security options

## 🚀 Future Scalability Options

The current architecture provides a solid foundation for horizontal and vertical scaling. Here are potential enhancements:

### 1. **Thread Pool Per Port**

Instead of assigning a single Tokio task per port, implement a configurable thread pool for each worker:

```toml
[server]
threads_per_port = 4  # Each port runs with 4 internal threads
worker_ports = 4
base_port = 8447
```

**Benefits**:
- Better CPU utilization through parallel request handling
- Improved throughput under high concurrency
- Fine-grained control over resource allocation per port


### 2. **Advanced Load Balancing**

Enhance the current round-robin algorithm:

- **Least Connections**: Route to the worker with fewest active connections
- **Weighted Round-Robin**: Assign different weights to workers based on capacity
- **Response Time-Based**: Route to fastest responding servers
- **Sticky Sessions**: Route user requests to the same worker for session consistency

### 4. **Caching Layer**

Introduce in-memory and distributed caching:

- **Redis Integration**: Cache frequently accessed data (user profiles, tokens)
- **In-Memory Cache**: Use libraries like `moka` or `lru` for local caching
- **Cache Invalidation**: Implement TTL-based or event-driven cache updates


### 5. **Rate Limiting & DDoS Protection**

Implement rate limiting and request throttling:

- **Per-User Rate Limits**: Limit requests per user/IP address
- **Token Bucket Algorithm**: Implement token bucket for smooth rate limiting
- **Circuit Breaker**: Fail fast when service is overloaded
- **Request Batching**: Batch multiple requests to reduce processing overhead

### 6. **Async Authentication & Authorization**

Optimize the current RBAC implementation:

- **JWT Caching**: Cache token validation results temporarily
- **Permission Caching**: Cache role-based permissions with TTL
- **Async Token Refresh**: Background token refresh before expiration
- **Multi-Tenant Support**: Extend RBAC for multiple organizations


These enhancements will transform the server from handling thousands of concurrent connections to millions while maintaining low latency and high availability.

## 📚Authors & Contributors
Developed by **Team RustyGate**:
- Aman Revankar
- Vineet
- Pal Patel
- Srishti
- Jeferson
