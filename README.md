# 🌐 Local Server API (Rust+Warp+PostgreSQLsqlx)

This is a **Local Server API** project built with **Rust**, using the **Warp** framework for the web server and **SQLx** for managing the PostgreSQL database.

---

## ▶️ Features
- User registration and login and delete
- Password security using **Argon2**
- PostgreSQL database connection via **SQLx**
- Basic API routes for a backend server
- Security Features: JWT Authentication, Idle Timeout, Rate Limiting

## ▶️ Run the App
1. **Clone the repository**
    - `git clone https://github.com/NguyenKhanhHoa6235/Local-server-API---Rust_Warp_Postgres_sqlx.git`
    - `cd Local-server-API---Rust_Warp_Postgres_sqlx`

2. **Create a `.env` file in the project root**
    - `DATABASE_URL=postgres://username:password@localhost:5432/dbname`
    - `RUST_LOG=info`
    - `BIND_ADDRESS=127.0.0.1`
    - `JWT_SECRET=your_super_secret_key`
    - **Note:** Replace `username`, `password`, `dbname` with your PostgreSQL credentials. `JWT_SECRET` is used to sign and verify JWT tokens.

3. **Install SQLx CLI** (to run migrations)
    ```bash
    cargo install sqlx-cli --no-default-features --features postgres
    ```

4. **Run migrations**
    ```bash
    sqlx migrate run
    ```

5. **Prepare SQLx for offline queries (optional)**
    ```bash
    cargo sqlx prepare
    ```

6. **Run the server**
    ```bash
    cargo run
    ```
    - The server will run at: `http://127.0.0.1:3030`


### 🔹 Explanation
- **Cargo.toml**: Declares project dependencies and metadata.  
- **.env**: Stores environment variables like `DATABASE_URL`, `BIND_PORT`, etc.  
- **migrations/**: Contains SQL files to create tables and manage database schema.  
- **src/main.rs**: Starts the Warp server and initializes DB connection.  
- **src/routes.rs**: Defines HTTP routes and maps them to handlers.  
- **src/handlers.rs**: Contains functions that handle requests and responses.  
- **src/models.rs**: Defines data structures for requests, responses, and DB.  
- **src/db.rs**: Sets up database pool connection using SQLx.  
- **src/errors.rs**: Defines custom API error types.
- **src/jwt.rs**: JWT creation, verification, idle timeout tracking.
- **src/rate_limit.rs**: Rate limiting logic per IP.
