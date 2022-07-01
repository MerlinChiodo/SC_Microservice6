use actix_web::{http, HttpResponse, Responder, web};
use serde_json::Value;
use crate::server::{BackendServerInfo, DBPool};
use serde::Deserialize;
use actix_web::get;

pub(crate) async fn ping(config: web::Data<BackendServerInfo>) -> impl Responder {
    let body = serde_json::to_string(&config.info).unwrap();
    HttpResponse::Ok()
        .content_type(http::header::ContentType::json())
        .body(body)
}

/*
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
*/