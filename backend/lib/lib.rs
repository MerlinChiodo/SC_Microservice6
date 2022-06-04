#![feature(future_join, future_poll_fn)]
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



