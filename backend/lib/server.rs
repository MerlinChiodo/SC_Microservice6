
use std::fmt;
use diesel::prelude::*;
use moon::{
    actix_cors::Cors,
    actix_web::{
        HttpResponse,
        HttpServer,
        HttpRequest,
        body::MessageBody,
        Error,
        dev::{ServiceFactory, ServiceRequest, ServiceResponse},
        http::{header::ContentType, StatusCode},
        middleware::{Compat, Condition, ErrorHandlers, Logger},
        web::{self, ServiceConfig},
        App, Responder,
    },
    config::CONFIG,
    *,
};
use self::config::Config;
use std::fmt::{Display, format, Formatter, write};
use secrecy::Secret;
use serde::{Deserialize, Serialize};
use std::net::TcpListener;
use actix_web::web::route;
use diesel::r2d2::ConnectionManager;
use diesel_migrations::embed_migrations;
use crate::endpoints::{login_simple, register_simple, validate_token_simple};

pub type DBPool = diesel::r2d2::Pool<ConnectionManager<MysqlConnection>>;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerConfig {
    info: AuthServerInfo,
    db: Option<DatabaseInfo>,
    rmq: Option<RabbitMQServerInfo>
}
impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            info: AuthServerInfo::default(),
            db: None,
            rmq: None,
        }
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DatabaseInfo {
    host: String,
    username: String,
    password: String,
    name: String
}
impl ToString for DatabaseInfo {
    fn to_string(&self) -> String {
        format!("mysql://{}:{}@{}/{}",
                self.username, self.password,
                self.host, self.name)
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RabbitMQServerInfo {
    host: String,
    username: String,
    password: String,
}

impl ToString for RabbitMQServerInfo {
    fn to_string(&self) -> String {
        format!("amqp://{}:{}@{}",
                self.username, self.password,
                self.host)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthServerInfo {
    api_version: String,
    server_version: String,
}
impl Default for AuthServerInfo {
    fn default() -> Self {
        AuthServerInfo {
            api_version: String::from("Unknown Version"),
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

pub fn set_server_api_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/ping", web::get().to(ping))
        .route("/register", web::post().to(register_simple))
        .route("/login", web::post().to(login_simple))
        .route("/verify", web::post().to(validate_token_simple));
}

async fn up_msg_handler(_: UpMsgRequest<()>) {}

async fn frontend() -> Frontend {
    Frontend::new()
        .title("SmartAuth")
        .default_styles(false)
        .append_to_head(r#"<link rel="stylesheet" href="https://unpkg.com/@picocss/pico@latest/css/pico.min.css">"#)
        .append_to_head(r#"<link rel="stylesheet" href="/_api/public/custom.css">"#)
        .body_content(r#"<div id="app"></div>"#)
}

#[derive(Debug)]
pub enum ServerCreationError {
    DBError(DBConnectionError),
    RMQError(RMQConnectionError),
}

#[derive(Debug)]
pub enum DBConnectionError {
    MissingSettings,
    //TODO: This is bad, fix it!
    ConnectionError(String)
}
#[derive(Debug)]
pub enum RMQConnectionError {
    MissingSettings,
    ConnectionError(deadpool_lapin::BuildError)
}
pub fn connect_to_db(config: &ServerConfig) -> Result<DBPool, DBConnectionError> {
    let db_url = match &config.db {
        None => {
            std::env::var("DATABASE_URL")
                .map_err(|_| DBConnectionError::MissingSettings)?
        }
        Some(db) => {
            db.to_string()
        }
    };
    let db_manager = ConnectionManager::<MysqlConnection>::new(db_url);

     diesel::r2d2::Pool::builder().build(db_manager)
         .map_err(|err| DBConnectionError::ConnectionError(err.to_string()))
}

pub fn connect_to_rmq(config: &ServerConfig) -> Result<deadpool_lapin::Pool, RMQConnectionError> {
    let rmq_url = match &config.rmq {
        None => {
            std::env::var("AMQP_ADDR")
                .map_err(|_| RMQConnectionError::MissingSettings)?
        }
        Some(rmq) => {
            rmq.to_string()
        }
    };

    let rmq_connection_options = lapin::ConnectionProperties::default()
        .with_executor(tokio_executor_trait::Tokio::current());

    let rmq_manager = deadpool_lapin::Manager::new(rmq_url, rmq_connection_options);

    deadpool::managed::Pool::builder(rmq_manager)
        .max_size(10)
        .build()
        .map_err(|err| RMQConnectionError::ConnectionError(err))
}

pub async fn server_start(config: ServerConfig, listener: TcpListener) -> Result<(), ServerCreationError>{
    let db_pool = connect_to_db(&config)
        .map_err(|err| ServerCreationError::DBError(err))?;

    let rmq_pool = connect_to_rmq(&config)
        .map_err(|err| ServerCreationError::RMQError(err))?;

    let app = move || {
        let redirect = Redirect::new()
            .http_to_https(CONFIG.https)
            .port(CONFIG.redirect.port, CONFIG.port);// TODO: Check if we have a port, otherwise assign a random one

        App::new()
            .wrap(Condition::new(CONFIG.redirect.enabled, Compat::new(redirect)))
            .wrap(Logger::new("%r %s %D ms %a"))
            .wrap(Cors::default().allowed_origin_fn(move |origin, _| {
                if CONFIG.cors.origins.contains("*") {
                    return true;
                }
                let origin = match origin.to_str() {
                    Ok(origin) => origin,
                    Err(_) => return false,
                };
                CONFIG.cors.origins.contains(origin)
            }))

            .wrap(ErrorHandlers::new().handler(StatusCode::INTERNAL_SERVER_ERROR, error_handler::internal_server_error)
                .handler(StatusCode::NOT_FOUND, error_handler::not_found))

            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(rmq_pool.clone()))
    };

    start_with_app(frontend, up_msg_handler, app, set_server_api_routes).await.unwrap();
    Ok(())
}

async fn ping(config: web::Data<ServerConfig>) -> impl Responder {
    let body = serde_json::to_string(&config.info).unwrap();
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(body)
}
