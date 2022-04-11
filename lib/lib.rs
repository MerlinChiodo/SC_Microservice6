use actix_web::{
    body::BoxBody, cookie::time::format_description::modifier::Second, dev::Server,
    http::header::ContentType, web, App, CustomizeResponder, HttpRequest, HttpResponse, HttpServer,
    Responder,
};

use actix_web::http::header::TryIntoHeaderPair;
use actix_web::http::StatusCode;
use config::Config;
use core::fmt;
use secrecy::Secret;
use serde::{Deserialize, Serialize};
use std::net::TcpListener;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerConfig {
    info: AuthServerInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthServerInfo {
    api_version: String,
    server_version: String,
}
impl Default for AuthServerInfo {
    fn default() -> Self {
        AuthServerInfo {
            api_version: String::from("Unknown Version"), //TODO: Read the api version from somewhere
            server_version: String::from(
                option_env!("CARGO_PKG_VERSION").unwrap_or("Unknown Version"),
            ),
        }
    }
}

impl fmt::Display for AuthServerInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(API: {}, Server: {})",
            self.api_version, self.server_version
        )
    }
}

struct UserRegisterData {
    username: Secret<String>,
    password: Secret<String>,
    mail: Secret<String>,
}

fn index(form: web::Form<UserRegisterData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}

async fn register() -> HttpResponse {
    HttpResponse::Ok().finish()
}
async fn ping(config: web::Data<ServerConfig>) -> impl Responder {
    let body = serde_json::to_string(&config.info).unwrap();
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(body)
}

pub fn server_start(config: ServerConfig, listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(config.clone()))
            .route("/ping", web::get().to(ping))
            .route("/users", web::post().to(register))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
