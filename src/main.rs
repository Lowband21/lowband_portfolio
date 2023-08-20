//! main.rs
use actix_session::storage::RedisSessionStore;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;
use lowband_portfolio::configuration::get_configuration;
use lowband_portfolio::startup::run;
use lowband_portfolio::DbPool;
use std::env;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    //let configuration = get_configuration().expect("Failed to read configuration.");
    let port = env::var("PORT").unwrap_or("5000".to_string());

    let address = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(address)?;

    let redis_connection_string =
        std::env::var("REDIS_URL").unwrap_or_else(|_| String::from("redis://127.0.0.1:6379"));
    let redis_store = RedisSessionStore::new(redis_connection_string)
        .await
        .unwrap();

    //let connection_string = configuration.database.connection_string();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    // The `Connection` trait MUST be in scope for us to invoke
    // `PgConnection::connect` - it is not an inherent method of the struct!
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool: DbPool = Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    run(listener, redis_store, pool)?.await
}
