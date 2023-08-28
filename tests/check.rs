use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;
use demcru::configuration::{get_config, DatabaseSettings};

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {

    let client = reqwest::Client::new();
    let response = client.get("localhost:8080/")
        .send()
        .await
        .expect("Failed to execute request");
    assert!(response.status().is_success());

    let saved = sqlx::query!("SELECT * FROM contacts")
        
        

}
