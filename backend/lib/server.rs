mod routes;

use std::fmt;
use actix_web::{App, web};
use actix_web::http::StatusCode;
use actix_web::middleware::{Compat, Condition, ErrorHandlers, Logger};
use either::Either;
use anyhow::{Context, ensure, Result};
use config::Config;
use diesel::MysqlConnection;
use diesel::r2d2::ConnectionManager;
use lapin::message::Delivery;
use lapin::options::{BasicAckOptions, BasicConsumeOptions};
use lapin::types::FieldTable;
use lettre::smtp::authentication::Credentials;
use lettre::SmtpClient;
use log::{debug, info};
use moon::actix_cors::Cors;
use moon::config::{CONFIG};
use moon::{error_handler, Frontend, Redirect};
use std::future::join;
use std::io::Write;

use moon::start_with_app;
use moon::futures::StreamExt;
use std::str;
pub type DBPool = diesel::r2d2::Pool<ConnectionManager<MysqlConnection>>;
pub type RMQPool = deadpool_lapin::Pool;

use serde::{Serialize, Deserialize};
use serde_json::Value;
use crate::auth::Actions::{insert_new_pending_user, login_employee, register_employee, send_citizen_code};
use crate::auth::Citizen::{Citizen, IsCitizen};
use crate::auth::Endpoints::{employee_login, employee_login_external, employee_register, employee_verify, login_external, login_page, user_login, user_register, user_verify};
use crate::server::routes::{ping};

#[derive(Clone)]
pub struct MailServer {
    pub info: ServerCredentials,
    pub transport: SmtpClient
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BackendServerInfo {
    info: AuthServerInfo,
    #[serde(with = "either::serde_untagged")]
    db: Either<ServerCredentials, String>,
    #[serde(with = "either::serde_untagged")]
    rmq: Either<ServerCredentials, String>,
    mail: ServerCredentials,
}
impl BackendServerInfo {
    fn try_from_file(path: &str) -> Result<Self> {
        debug!("Trying to read config file from {}", path);
        let server_config_file = Config::builder()
            .add_source(config::File::with_name(path))
            .build()
            .context("Failed to retrieve config file")?;

        debug!("Config file found");
        Ok(server_config_file
            .try_deserialize::<Self>()
            .context("Failed to parse config file")?)
    }

    fn try_from_env() -> Result<Self> {
        debug!("Trying to read config from env variables");
        Ok(Self
        {
            info: AuthServerInfo::default(),
            db: Either::Right(std::env::var("DATABASE_URL")?),
            rmq: Either::Right(std::env::var("AMQP_ADDR")?),
            mail: ServerCredentials {
                host: std::env::var("MAIL_HOST")?,
                username: std::env::var("MAIL_USERNAME")?,
                password: std::env::var("MAIL_PASSWORD")?
            }
        })
    }
}

#[derive(Clone)]
pub struct BackendServer {
    info: BackendServerInfo,
    db_pool: DBPool,
    rmq_pool: RMQPool,
    mail_sender: MailServer
}

impl BackendServer {
    pub fn new(config_path: Option<&str>) -> Result<Self> {
        println!("Reading config file...");

        let info =
            match config_path {
                None => {BackendServerInfo::try_from_env()?}
                Some(p) => {BackendServerInfo::try_from_file(p).or_else(|_|BackendServerInfo::try_from_env())?}
            };
        println!("... done");

        println!("Connecting to database...");
        let db_pool = Self::connect_to_database(&info)?;
        println!("...done!");

        println!("Connecting to rmq...");
        let rmq_pool = Self::connect_to_rmq(&info)?;
        println!("...done!");

        println!("Creating a mail sender...");
        let mail_sender = Self::create_mail_sender(&info)?;
        println!("...done!");

        Ok(Self{
            info,
            db_pool,
            rmq_pool,
            mail_sender
        })
    }

    pub async fn start(config_path: Option<&str>) -> Result<()> {
        let server = BackendServer::new(config_path)?;
        let rmq_server = server.clone();
        let rmq_thread = rmq_server.events_listen(5);

        let query_cfg = web::QueryConfig::default()
            .error_handler(|err, req| {
                actix_web::error::InternalError::from_response(
                    "",
                    actix_web::HttpResponse::BadRequest()
                        .content_type("application/json")
                        .body(format!(r#"{{"error": "{:?}"}}"#, err)),
                ).into()
            });

        let form_cfg= web::FormConfig::default()
            .error_handler(|err, req| {
                actix_web::error::InternalError::from_response(
                    "",
                    actix_web::HttpResponse::BadRequest()
                        .content_type("application/json")
                        .body(format!(r#"{{"error": "{:?}"}}"#, err)),
                ).into()
            });

        info!("Starting the server");

        let app = move || {
            let redirect = Redirect::new()
                .http_to_https(CONFIG.https)
                .port(CONFIG.redirect.port, CONFIG.port);

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

                .app_data(query_cfg)
                .app_data(web::FormConfig::default().error_handler(|err, req| {
                    actix_web::error::InternalError::from_response(
                        "",
                        actix_web::HttpResponse::BadRequest()
                            .content_type("application/json")
                            .body(format!(r#"{{"error": "{:?}"}}"#, err)),
                    ).into()
                }))
                .app_data(web::Data::new(server.db_pool.clone()))
        };
        let server_thread = async {start_with_app(Self::frontend, Self::up_msg_handler, app, Self::set_routes).await.unwrap() };
        join!(server_thread, rmq_thread).await;
        info!("Server done!");
        Ok(())
    }

    fn set_routes(cfg: &mut web::ServiceConfig) {
        /*
        cfg.route("/ping", web::get().to(ping))
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
            .route("/verify", web::post().to(validate_token_simple))
            .route("/test", web::get().to(|| async {"Hey"}))
            .route("/page/login", web::get().to(login_page))
            .route("/external", web::get().to(login_external))
            .service(on_login_test);

         */
        cfg.route("/ping", web::get().to(ping))
            .route("/login", web::post().to(user_login))
            .route("/verify", web::post().to(user_verify))
            .route("/register", web::post().to(user_register))
            .route("/external", web::get().to(login_external))
            .route("/employee/login", web::post().to(employee_login))
            .route("/employee/register", web::post().to(employee_register))
            .route("/employee/verify", web::post().to(employee_verify))
            .route("/page/login", web::get().to(login_page))
            .route("/employee/external", web::get().to(employee_login_external));

    }

    async fn up_msg_handler(_: moon::UpMsgRequest<()>) {}

    async fn events_handle(&self) -> Result<()> {
        info!("Listening to events");
        let connection = self.rmq_pool.get().await?;

        let channel = connection.create_channel().await?;
        let mut consumer = channel.basic_consume("smartauth", "new_citizen_consumer", BasicConsumeOptions::default(), FieldTable::default()).await?;

        while let Some(message) = consumer.next().await {
            info!("Got an event!");
            let message: Delivery = message?;

            message
                .ack(BasicAckOptions::default())
                .await?;
            debug!("Message body {:?}", str::from_utf8(&message.data)?);

            let new_citizen_event_id = 1001;
            let json_data: Value = serde_json::from_slice(&message.data)?;
            if json_data["event_id"] == 1001 {
                println!("We got an event");
                let id = json_data.get("citizen_id");
                ensure!(id.is_some());
                let id = id.unwrap().as_i64();
                ensure!(id.is_some());
                let db = self.db_pool.get()?;

                let code = insert_new_pending_user(&db, id.unwrap())?;
                let citizen = Citizen {
                    citizen_id: id.unwrap() as u64
                };

                let info = citizen.get_citizen_info().await?;
                println!("Got citizen information");
                send_citizen_code(&self.mail_sender.transport, &info, &code).await?;
            }
        }
        Ok(())
    }

    async fn events_listen(&self, poll_interval_secs: u64) -> Result<()> {
        let mut retry = tokio::time::interval(std::time::Duration::from_secs(poll_interval_secs));

        loop {
            retry.tick().await;
            match self.events_handle().await {
                Ok(_) => debug!("Got a message and handled it without error"),
                Err(e) => debug!("Got a message but failed with error: {:?}", e)
            };
        }
    }

    fn connect_to_database(config: &BackendServerInfo) -> Result<DBPool> {
        let db_url = &config.db.as_ref().either(|l| format!("mysql://{}:{}@{}/{}", l.username, l.password, l.host, "SmartAuth"), |r| r.clone());
        info!("Got a database url: {}", db_url);
        let db_manager = ConnectionManager::<MysqlConnection>::new(db_url);
        Ok(diesel::r2d2::Pool::builder().build(db_manager)?)
    }

    fn connect_to_rmq(config: &BackendServerInfo) -> Result<RMQPool> {
        let rmq_url = config.rmq.as_ref().either(|l| format!("amqp://{}:{}@{}", l.username, l.password, l.host), |r| r.clone());
        let rmq_connection_options = lapin::ConnectionProperties::default()
            .with_executor(tokio_executor_trait::Tokio::current());

        let rmq_manager = deadpool_lapin::Manager::new(rmq_url, rmq_connection_options);

        Ok(deadpool::managed::Pool::builder(rmq_manager)
            .max_size(10)
            .build()?)
    }

    fn create_mail_sender(config: &BackendServerInfo) -> Result<MailServer> {
        Ok(MailServer {
            transport: SmtpClient::new_simple(config.mail.host.as_str())?
                .credentials(Credentials::new(config.mail.username.clone(), config.mail.password.clone())),

            info: config.mail.clone(),
        })
    }
    async fn frontend() -> Frontend {
        Frontend::new()
            .title("SmartAuth")
            .default_styles(false)
            .append_to_head(r#"<link rel="stylesheet" href="https://unpkg.com/@picocss/pico@latest/css/pico.min.css">"#)
            .append_to_head(r#"<link rel="stylesheet" href="/_api/public/custom.css">"#)
            .body_content(r#"<div id="app"></div>"#)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerCredentials {
    host: String,
    username: String,
    password: String
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
