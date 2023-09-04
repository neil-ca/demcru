use demcru::configuration::get_config;
use demcru::startup;
use sqlx::{Connection, PgConnection, PgPool};
use std::net::TcpListener;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

fn spawn_app() -> TestApp {
    let config = get_config().expect("Failed to read config");
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let connection_pool =
        PgPool::connect_lazy(&config.database.connection_string()).expect("failed.");
    let server = startup::run(listener, connection_pool.clone()).expect("Failed to bind address");
    let _ = tokio::spawn(server);
    TestApp {
        address,
        db_pool: connection_pool,
    }
}

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app();

    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/health-check", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
}
#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app = spawn_app();
    let config = get_config().expect("Fail read config");
    let conn_str = config.database.connection_string();
    let mut conn = PgConnection::connect(&conn_str)
        .await
        .expect("Failed to connect to postgres");
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/contacts", &addr))
        .send()
        .await
        .expect("failed request");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT * FROM contacts")
        .fetch_all(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");
    assert!(saved);
}
