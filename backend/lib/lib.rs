#![feature(future_join, future_poll_fn)]
#![feature(bool_to_option)]
#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

pub mod auth;
pub mod server;
pub mod schema;



