use actix_web::{
    body::BoxBody,
    dev::Server,
    http::{header::{ContentType}},
    web, App, HttpRequest, HttpResponse, HttpServer, Responder, cookie::time::format_description::modifier::Second,
};

use serde::Serialize;
use core::fmt;
use std::{net::TcpListener};

#[derive(Serialize, Debug)]
struct AuthServerInfo<'a> {
    api_version: &'a str,
    server_version: &'a str,
}

impl<'a> Responder for AuthServerInfo<'a> {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        let body = serde_json::to_string(&self).unwrap();
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(body)
    }
}

impl<'a> Default for AuthServerInfo<'a> {
    fn default() -> Self {
        AuthServerInfo {
            api_version: "0.01", //TODO: Read the api version from somewhere
            server_version: option_env!("CARGO_PKG_VERSION").unwrap_or("Unknown Version"),
        }
    }
}
impl<'a> fmt::Display for AuthServerInfo<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(API: {}, Server: {})", self.api_version, self.server_version)
    }
}

async fn ping(_req: HttpRequest) -> impl Responder {
    AuthServerInfo::default()
}

pub fn server_start(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| App::new().route("ping", web::get().to(ping)))
        .listen(listener)?
        .run();

    Ok(server)
}
