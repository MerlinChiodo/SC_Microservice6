#![feature(future_join, future_poll_fn)]
#![feature(bool_to_option)]
#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

pub mod error;
pub mod auth;
pub mod server;
pub mod schema;
pub mod actions;
pub mod models;
pub mod endpoints;
pub mod user;
pub mod request;
pub mod session;
pub mod employee;
pub mod microservices;



