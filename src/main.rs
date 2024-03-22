use tokio::net::TcpListener;
use std::net::SocketAddr;
use std::str::FromStr;
use router::create_router;

mod api;
mod error;
mod router;
mod todo;

async fn init_dbpool() -> Result<sqlx::Pool<sqlx::Sqlite>, sqlx::Error> {
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

    // We'll try to read the DATABASE_URL environment variable or default sqlite:db.sqlite if not defined
    // (Which opens a file called db.sqlite in the current working directory)
    let db_connection_str = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:db.sqlite".to_string());

    // When we connect to the database, we ask the driver to create the database if it doesn't already exit.
    let db_pool = SqlitePoolOptions::new()
        .connect_with(SqliteConnectOptions::from_str(&db_connection_str)?
            // SQLx will generate a `CREATE DATABASE IF NOT EXISTS` for us
                          .create_if_missing(true))
        .await
        .expect("can't connect to database");

    // After we've connected to the DB, we run any necessary migrations.
    sqlx::migrate!()
        // We can pass our newly created DB pool directly to SQLx, which will obtain a connection from the pool.
        .run(&db_pool)
        .await
        .expect("database migration failed");
    Ok(db_pool)
}

fn init_tracing() {
    use tracing_subscriber::{
        filter::LevelFilter, fmt, prelude::*, EnvFilter
    };

    // Fetches the RUST_LOG environment providing a default value if it's not defined
    let rust_log = std::env::var(EnvFilter::DEFAULT_ENV)
        .unwrap_or_else(|_| "sqlx=info,tower_http=debug,info".to_string());

    // Returns the default global registry
    tracing_subscriber::registry()
        // Adds a formatting layer, which provides human-readable trace formatting
        .with(fmt::layer())
        // Constructs an environment filter, with the default log level set to info or using the
        // value provided by RUST_LOG otherwise
        .with(EnvFilter::builder()
            .with_default_directive(LevelFilter::INFO.into())
            .parse_lossy(rust_log),
        ).init();
}

#[tokio::main]
async fn main() {
    // Initializes the tracing and logging for our service and its dependencies
    init_tracing();

    // Initializes the DB pool
    let dbpool = init_dbpool().await
        .expect("couldn't initialize DB pool");

    // Creates the core application service and its routes
    let router = create_router(dbpool).await;

    // Fetches the binding address from the environment variable
    // BIND_ADDR or uses the default value of 127.0.0.1:3000
    let bind_addr = std::env::var("BIND_ARRD")
        .unwrap_or_else(|_| "127.0.0.1:3000".to_string());

    let addr = SocketAddr::from_str(&bind_addr).unwrap();
    let tcp = TcpListener::bind(&addr).await.unwrap();

    // Parses the binding address into socket address

    axum::
        // Creates the service and starts the HTTP server
        serve(tcp,router.into_make_service())
        .await
        .expect("unable to start server");
}
