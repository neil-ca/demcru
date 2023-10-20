use demcru::configuration::get_config;
use sqlx::{PgConnection, Connection};

use crate::helpers::spawn_app;


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
        .get(&format!("{}/contacts", &app.address))
        .send()
        .await
        .expect("failed request");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT * FROM contacts")
        .fetch_all(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");
}
