use backend::*;
use moon::main;
use config::Config;

#[moon::main]
async fn main() -> std::io::Result<()> {
    let server_config = Config::builder()
        .add_source(config::File::with_name("config/server.toml"))
        .build()
        .unwrap();

    let server_config = server_config.try_deserialize::<server::ServerConfig>().unwrap();

    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    println!("{}",port);

    server::server_start(server_config, listener).await;
    Ok(())
}
