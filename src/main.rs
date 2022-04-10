use actix_web::{get, http::Method, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use std::net::TcpListener;
use SmartCity_Auth::server_start;
#[actix_web::main]

async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();

    let port = listener.local_addr().unwrap().port();

    server_start(listener)?.await
}
