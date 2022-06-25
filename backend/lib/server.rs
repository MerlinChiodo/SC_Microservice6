use std::string::String;
use std::{fmt, time};
use diesel::prelude::*;
use actix_web::{dev, error, get, http, HttpResponseBuilder, ResponseError};
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
use std::future::join;
use secrecy::Secret;
use serde::{Deserialize, Serialize};
use std::net::TcpListener;
use actix_web::web::{Bytes, BytesMut, route};
use diesel::r2d2::ConnectionManager;
use diesel_migrations::embed_migrations;
use lapin::message::Delivery;
use lapin::options::{BasicAckOptions, BasicConsumeOptions};
use lapin::types::FieldTable;
use moon::futures::{SinkExt, StreamExt};
use serde_json::{json, Number, Value};
use crate::endpoints::{employee_login, employee_login_external, employee_register, employee_verify, login, login_external, login_page, login_simple, register, validate_token_simple};

pub type DBPool = diesel::r2d2::Pool<ConnectionManager<MysqlConnection>>;
pub type RMQPool = deadpool::managed::Object<deadpool_lapin::Manager>;
use std::str;
use actix_web::body::to_bytes;
use actix_web::dev::{AnyBody, Body, ResponseBody};
use actix_web::http::header;
use actix_web::middleware::ErrorHandlerResponse;
use deadpool::managed::{Object, PoolError};
use deadpool_lapin::{Manager, Pool};
use diesel::r2d2;
use lettre::{smtp, SmtpClient, SmtpTransport};
use lettre::smtp::authentication::Credentials;
use crate::actions::{check_token, insert_new_pending_user, send_citizen_code};
use crate::server::RmqError::LapinError;
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerConfig {
    info: AuthServerInfo,
    db: Option<DatabaseInfo>,
    rmq: Option<RabbitMQServerInfo>,
    mail: Option<MailServerInfo>
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            info: AuthServerInfo::default(),
            db: None,
            rmq: None,
            mail: None
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MailServerInfo {
    username: String,
    host: String,
    password: String
}

impl MailServerInfo {
    pub fn to_credentials(&self) -> smtp::authentication::Credentials {
        Credentials::new(self.username.clone(), self.password.clone())
    }
}

#[derive(Clone)]
pub struct MailServer {
    pub info: MailServerInfo,
    pub transport: SmtpClient
}
#[derive(Deserialize)]
pub struct TokenQuery {
    token: String
}

#[get("/onLogin")]
pub async fn on_login_test(pool: web::Data<DBPool>, token: web::Query<TokenQuery>) -> impl Responder {
    println!("Testing login");
    let user_token = token.into_inner().token;
    let closure = {
        let t = user_token.clone();
        let db = pool.get().expect("Unable to get db connection");
        println!("Checking token, token is: {}", t);
        check_token(&db, &t)
    };

    let user = web::block(|| closure)
        .await
        .unwrap()
        .unwrap();


    println!("Trying to get information about user {} with id {}", user.username, user.id);
    let user_info = reqwest::get(format!("http://www.smartcityproject.net:9710/api/citizen/{}", user.id))
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    println!("{}", user_info);
    let json_data: Value = serde_json::from_str(&user_info).unwrap();

    format!("Hey {} {}, nice to meet you!", json_data.get("firstname").unwrap(), json_data.get("lastname").unwrap())
}

pub fn set_server_api_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/ping", web::get().to(ping))
        .route("/register", web::post().to(register))
        .route("/login", web::post().to(login))
        .route("/verify", web::post().to(validate_token_simple))
        .route("/test", web::get().to(|| async {"Hey"}))
        .route("/page/login", web::get().to(login_page))
        .route("/external", web::get().to(login_external))
        .route("/employee/register", web::post().to(employee_register))
        .route("/employee/login", web::post().to(employee_login))
        .route("/employee/verify", web::post().to(employee_verify))
        .route("/employee/external", web::get().to(employee_login_external))
        .service(on_login_test);
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
    MailError(MailError)
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

#[derive(Debug)]
pub enum MailError {
    MissingSettings,
    InvalidHost(lettre::smtp::error::Error)
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

pub fn create_mail_sender(config: &ServerConfig) -> Result<MailServer, MailError> {
    let (host, username, password) = match &config.mail {
        None => {
            ((std::env::var("MAIL_HOST").map_err(|_| MailError::MissingSettings)?,
              std::env::var("MAIL_USERNAME").map_err(|_| MailError::MissingSettings)?,
              std::env::var("MAIL_PASSWORD").map_err(|_| MailError::MissingSettings)?))
        }
        Some(mail) => {
            (mail.host.clone(), mail.username.clone(), mail.password.clone())
        }
    };

    Ok(MailServer {
        transport: SmtpClient::new_simple(&host.clone())
            .map_err(|e|MailError::InvalidHost(e))?
            .credentials(Credentials::new(username.clone(), password.clone())),

        info: MailServerInfo {
            username,
            host,
            password
        },
    })

}

pub enum RmqError {
    PoolError,
    LapinError(lapin::Error),
    MessageParseError,
    DBError(diesel::result::Error),
    DBPoolError
}

pub async fn rmg_handle_messages(mail: &MailServer, db_pool: DBPool, pool: deadpool_lapin::Pool, queue_name: &str, consumer_name: &str) -> Result<(), RmqError> {
    println!("RMQ: Listen...");

    //TODO: Add proper error handling
    let connection = pool.get()
        .await
        .map_err(|_| RmqError::PoolError)?;
    let channel = connection.create_channel().await.map_err(|e| RmqError::LapinError((e)))?;
    let mut consumer = channel.basic_consume(queue_name,
                                             consumer_name,
                                             BasicConsumeOptions::default(),
                                             FieldTable::default())
        .await
        .map_err(|e| RmqError::LapinError(e))?;

    while let(Some(message)) = consumer.next().await {
        println!("Got something!");
        let message: Delivery = message
            .map_err(|_| RmqError::MessageParseError)?;

        message
            .ack(BasicAckOptions::default())
            .await
            .map_err(|e| RmqError::LapinError(e))?;

        println!("{:?}", str::from_utf8(&message.data)
            .map_err(|_| RmqError::MessageParseError)?);

        let json_data: serde_json::Result<Value> = serde_json::from_slice(&message.data);

        if let Ok(json) = json_data {
            println!("{}",&json);
            if json["event_id"] == 1001 {
                println!("We got a new citizen");
                let id = json.get("citizen_id").ok_or(RmqError::MessageParseError)?;
                let db = &db_pool
                    .get()
                    .map_err(|_| RmqError::DBPoolError)?;
                let pending_user = insert_new_pending_user(&db,id.as_u64().unwrap())
                    .map_err(|e| RmqError::DBError(e))?;
                send_citizen_code(&mail.transport, &pending_user).await;
            }
        }
        //TODO Create pending citizen reg request
    };
    Ok(())
}

pub async fn rmg_listen(mail: &MailServer, db_pool: DBPool, pool: deadpool_lapin::Pool) -> Result<(), lapin::Error> {
    let mut retry = tokio::time::interval(time::Duration::from_secs(5));
    loop {
        retry.tick().await;
        match rmg_handle_messages(mail, db_pool.clone(), pool.clone(), "smartauth", "new_citizen_consumer").await {
            Ok(_) => println!("Got message success"),
            Err(e) => println!("rmq: uh oh")
        };
    }
}
trait BodyTest {
    fn as_str(&self) -> &str;
}

impl BodyTest for Bytes {
    fn as_str(&self) -> &str {
        std::str::from_utf8(self).unwrap()
    }
}
fn render_500<B>(mut res: dev::ServiceResponse<B>) -> actix_web::Result<ErrorHandlerResponse<B>> {
    /*
    let req = res.request();
    let res = res.map_body(|_, _| ResponseBody::Body(Body::from("Hey")).into_body());
    */
    let error_message: String = match res.response().error() {
        Some(e) => format!("{}", json!({"type": "request", "error": e.to_string()})),
        None => String::from("Unknown")
    };


    let new_body=  HttpResponse::BadRequest().json(error_message).into_body();

    res.headers_mut()
        .insert(http::header::CONTENT_TYPE, http::HeaderValue::from_static("application/json"));

    Ok(ErrorHandlerResponse::Response(res))
}

pub async fn server_start(config: ServerConfig, listener: TcpListener) -> Result<(), ServerCreationError>{
    let db_pool = connect_to_db(&config)
        .map_err(|err| ServerCreationError::DBError(err))?;

    let rmq_pool = connect_to_rmq(&config)
        .map_err(|err| ServerCreationError::RMQError(err))?;

    let mail_server = create_mail_sender(&config)
        .map_err(|err| ServerCreationError::MailError(err))?;

    let rmq_thread = rmg_listen(&mail_server, db_pool.clone(), rmq_pool.clone());
    let app = move || {
        let redirect = Redirect::new()
            .http_to_https(CONFIG.https)
            .port(CONFIG.redirect.port, CONFIG.port);// TODO: Check if we have a port, otherwise assign a random one

        App::new()
            .wrap(Condition::new(CONFIG.redirect.enabled, Compat::new(redirect)))
            .wrap(Logger::new("%r %s %D ms %a"))
            .wrap(ErrorHandlers::new().handler(StatusCode::BAD_REQUEST, render_500))
            .app_data(web::JsonConfig::default().error_handler(|err, _req| {
                error::InternalError::from_response(
                    "",
                    HttpResponse::BadRequest()
                        .content_type("application/json")
                        .body(format!(r#"{{"error": "{}"}}"#, err)),
                ).into()
            }))
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


            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(db_pool.clone()))
    };

    join!(async {start_with_app(frontend, up_msg_handler, app, set_server_api_routes).await.unwrap()}, rmq_thread).await;

    rmq_pool.close();
    Ok(())
}

async fn ping(config: web::Data<ServerConfig>) -> impl Responder {
    let body = serde_json::to_string(&config.info).unwrap();
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(body)
}
