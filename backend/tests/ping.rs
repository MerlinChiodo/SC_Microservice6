use std::io::Write;
use config::Config;
use diesel::insert_into;
use moon::{
    config::CONFIG,
};
use rand::distributions::Alphanumeric;
use rand::Rng;

use backend::*;
use backend::actions::{check_token, get_session, get_user, insert_new_session, insert_new_user};
use backend::models::{User, UserIdentityInfo};
use backend::schema::Users::username;
use backend::server::{connect_to_db, server_start, BackendServer};

#[tokio::test]
async fn ping_works() {
    let server_config_file = Config::builder()
        .add_source(config::File::with_name("config/server.toml"))
        .build();

    let server_config = server_config_file
        .map(|f| f.try_deserialize::<BackendServer>().expect("Invalid config file"))
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

pub fn new_user() -> UserIdentityInfo {
    let user_name: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();
    let password: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();

    UserIdentityInfo {
        name: user_name,
        password
    }
}
fn debug_connect_to_db() -> server::DBPool {
    let server_config_file = Config::builder()
        .add_source(config::File::with_name("config/server.toml"))
        .build();

    let server_config = server_config_file
        .map(|f| f.try_deserialize::<BackendServer>().expect("Invalid config file"))
        .expect("Unable to parse config file");

    connect_to_db(&server_config).unwrap()

}
#[tokio::test]
async fn create_user_simple(){
    let db_pool = debug_connect_to_db();

    let user_info = new_user();
    insert_new_user(&db_pool.get().unwrap(), user_info).unwrap();
}

#[tokio::test]
async fn register_then_login_simple() {
    let db_pool = debug_connect_to_db();

    let user_info = new_user();
    insert_new_user(&db_pool.get().unwrap(), user_info.clone()).unwrap();

    let result_user = get_user(&db_pool.get().unwrap(), &user_info).unwrap();
    assert_eq!(result_user.username,user_info.name);
}

#[tokio::test]
async fn basic_token_creation_works() {
    let db_pool = debug_connect_to_db();

    let user_info = new_user();
    insert_new_user(&db_pool.get().unwrap(), user_info.clone()).unwrap();

    let result_user = get_user(&db_pool.get().unwrap(), &user_info).unwrap();
    assert_eq!(result_user.username,user_info.name);

    insert_new_session(&db_pool.get().unwrap(), &result_user).unwrap();
    let _ = get_session(&db_pool.get().unwrap(), &result_user).unwrap();
}

#[tokio::test]
async fn basic_token_auth_works() {
    let db_pool = debug_connect_to_db();

    let user_info = new_user();
    insert_new_user(&db_pool.get().unwrap(), user_info.clone()).unwrap();

    let result_user = get_user(&db_pool.get().unwrap(), &user_info).unwrap();
    assert_eq!(result_user.username,user_info.name);

    insert_new_session(&db_pool.get().unwrap(), &result_user).unwrap();
    let session = get_session(&db_pool.get().unwrap(), &result_user).unwrap();


    let session_user = check_token(&db_pool.get().unwrap(), &session.token).unwrap();

    assert_eq!(session_user.username, result_user.username)
}