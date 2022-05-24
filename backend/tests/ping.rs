use moon::{
    config::CONFIG,
    *,
};
use backend::*;

#[tokio::test]
async fn ping_works() {
    let server_address = format!("http://localhost:{}", CONFIG.port);
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/hello", server_address))
        .send()
        .await
        .expect("Unable to execute request");

    assert!(response.status().is_success());
    assert_ne!(Some(0), response.content_length());

    let text = response.text().await.expect("Unable to get text");
    println!("Respsone: {:?}", &text);
}