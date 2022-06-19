use moon::main;
use config::Config;
use backend::server;
use backend::server::BackendServer;
use std::thread;
use anyhow::Result;

#[moon::main]
async fn main() -> Result<()>{
    /*
    let server_config_file = Config::builder()
        .add_source(config::File::with_name("config/server.toml"))
        .build();

    let server_config = server_config_file
        .map(|f| f.try_deserialize::<BackendServer>().expect("Invalid config file"))
        .ok();

    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    println!("{}",port);
    */
    //let server: BackendServer = BackendServer::new(Some("config/server.toml"))?;

    BackendServer::start(Some("config/server.toml")).await?;
    Ok(())
}
