use sqlx::PgPool;
use std::net::TcpListener;
use demcru::configuration::get_config;
use demcru::startup;
fn spawn_app() -> String {
    let config = get_config().expect("Failed to read config");
    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let connection_pool = PgPool::connect_lazy(&config.database.connection_string())
        .expect("failed.");
    let server = startup::run(listener, connection_pool).expect("Failed to bind address");
    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{}", port)
}

#[tokio::test]
async fn health_check_works() {
    let address = spawn_app();
    let client = reqwest::Client::new();
    let response = client 
        .get(&format!("{}/health-check", &address))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
}
// #[tokio::test]
// async fn subscribe_returns_a_200_for_valid_form_data() {
//
//     let client = reqwest::Client::new();
//     let response = client.get("localhost:8080/")
//         .send()
//         .await
//         .expect("Failed to execute request");
//     assert!(response.status().is_success());
//
//     let saved = sqlx::query!("SELECT * FROM contacts")
//        
//        
//
// }
