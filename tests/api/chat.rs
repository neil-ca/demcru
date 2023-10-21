use crate::helpers::spawn_app;
#[tokio::test]
async fn requests_missing_authorization_are_rejected() {
    let app = spawn_app().await;
    let response = reqwest::Client::new()
        .post(&format!("{}/chat-with-me", &app.address))
        .json(&serde_json::json!({
            "message": "Hey whats app",
        }))
        .send()
        .await
        .expect("Filed to execute request");
}
