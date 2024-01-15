use std::net::TcpListener;
use demcru::startup::run;
use demcru::configuration::get_config;
// use libsql_client::Client;
use sqlx::sqlite::SqlitePool;
use std::env;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let config = get_config().expect("Failed to read config");
    let connection_pool = SqlitePool::connect(&env::var("DATABASE_URL")?)
        .await
       .expect("Failed to connect to sqlite.");

    // let _db = Client::from_env().await.unwrap();
    let address = format!("0.0.0.0:{}", config.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool)?.await?;
    Ok(())
}
