use std::io::Write;
use config::Config;
use diesel::insert_into;
use moon::{
    config::CONFIG,
};

use backend::*;
use backend::actions::insert_new_user;
use backend::models::UserInfo;
use backend::server::{connect_to_db, server_start, ServerConfig};

#[tokio::test]
async fn ping_works() {
    let server_config_file = Config::builder()
        .add_source(config::File::with_name("config/server.toml"))
        .build();

    let server_config = server_config_file
        .map(|f| f.try_deserialize::<ServerConfig>().expect("Invalid config file"))
        .ok();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();

    let server_result = server_start(server_config.unwrap_or_default(), listener);

    let server_address = format!("http://localhost:{}", CONFIG.port);
    println!("{}", server_address);
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/ping", server_address))
        .send()
        .await
        .expect("Unable to execute request");

    assert!(response.status().is_success());
    assert_ne!(Some(0), response.content_length());

    let text = response.text().await.expect("Unable to get text");
    println!("Respsone: {:?}", &text);
}

#[tokio::test]
async fn create_user_simple() {
    let server_config_file = Config::builder()
        .add_source(config::File::with_name("config/server.toml"))
        .build();

    let server_config = server_config_file
        .map(|f| f.try_deserialize::<ServerConfig>().expect("Invalid config file"))
        .expect("Unable to parse config file");

    let db_pool = connect_to_db(&server_config).unwrap();

    let user_info = UserInfo {
        name: "CraigAllanRothwell".to_string(),
        password: "SuperSecret123".to_string(),
    };

    insert_new_user(&db_pool.get().unwrap(), user_info).unwrap();

}
