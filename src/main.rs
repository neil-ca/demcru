use std::net::TcpListener;
use demcru::startup::run;

use demcru::configuration::get_config;
use sqlx::PgPool;
#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config = get_config().expect("Failed to read config");
    let connection_pool = PgPool::connect(&config.database.connection_string())
        .await
        .expect("Failed to connect to postgres.");
    let address = format!("127.0.0.1:{}", config.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool)?.await?;
    Ok(())
}


