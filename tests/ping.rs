use std::net::TcpListener;

use reqwest::Client;

#[tokio::test]
async fn ping_works() {
    let server_adress = spawn_server();
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/ping", server_adress))
        .send()
        .await
        .expect("Unable to execute request");

    assert!(response.status().is_success());
    assert_ne!(Some(0), response.content_length());

    let text = response.text().await.expect("Unable to get text");
    println!("Respsone: {:?}", &text);
}

fn spawn_server() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();

    let port = listener.local_addr().unwrap().port();

    let _ = tokio::spawn(SmartCity_Auth::server_start(listener).expect("Failed to start server"));
    format!("http://127.0.0.1:{}", port)
}
