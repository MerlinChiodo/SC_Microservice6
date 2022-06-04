use moon::main;
use config::Config;
use backend::server;
use backend::server::ServerConfig;
use std::thread;

#[moon::main]
async fn main() -> std::io::Result<()> {

    let server_config_file = Config::builder()
        .add_source(config::File::with_name("config/server.toml"))
        .build();

    let server_config = server_config_file
        .map(|f| f.try_deserialize::<ServerConfig>().expect("Invalid config file"))
        .ok();

    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    println!("{}",port);

    async {
        server::server_start(server_config.unwrap_or_default(), listener).await.unwrap();
    }.await;
    Ok(())
}
