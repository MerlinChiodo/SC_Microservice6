use actix_web::{get, http::Method, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use config::Config;
use std::net::TcpListener;
use SmartCity_Auth::{server_start, ServerConfig};
#[actix_web::main]

async fn main() -> std::io::Result<()> {
    let server_config = Config::builder()
        .add_source(config::File::with_name("config/server.toml"))
        .build()
        .unwrap();

    let server_config = server_config.try_deserialize::<ServerConfig>().unwrap();

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    server_start(server_config, listener)?.await
}
