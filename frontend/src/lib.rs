use rand::prelude::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::{array, iter::repeat_with, ops::Not};
use std::any::Any;
use zoon::{format, *};
use zoon::dominator::html;
use zoon::named_color::{GREEN_7, GREEN_8};
use zoon::Tag::{Article, H1, H2, Main};

pub struct UserInfo {
    username: String
}

fn page() -> impl Element {
    RawHtmlEl::new("body")
        .class("container")
        .child(
            RawHtmlEl::new("main")
                .class("container")
                .child(
                    RawHtmlEl::new("article")
                        .child(
                            RawHtmlEl::new("div")
                                .child(heading_group())
                                .child(register_form())
                        )
                )
        )
}

fn heading_group() -> impl Element {
    RawHtmlEl::new("hgroup")
        .child(RawHtmlEl::new("h1").child("Registrieren"))
        .child(RawHtmlEl::new("h2").child("Neuen Account erstellen"))
}

fn register_form() -> impl Element {
    RawHtmlEl::new("form")
        .attr("action", "/register")
        .attr("enctype", "application/x-www-form-urlencoded")
        .attr("method", "post")
        .child(
            RawHtmlEl::new("div")
                .class("grid")
                .child(username_input())
                .child(mail_input())
        )
        .child(cit_id_input())
        .child(pw_input())
        .child(submit_button())
}

pub struct UsernameInput {
    username: Mutable<String>,
    raw: RawHtmlEl,

}
fn username_input() -> impl Element {
    RawHtmlEl::new("input")
        .attr("type", "text")
        .attr("name", "name")
}
fn mail_input() -> impl Element {
    RawHtmlEl::new("input")
        .attr("type", "text")
        //.attr("name", "login")

}
fn cit_id_input() -> impl Element {
    RawHtmlEl::new("input")
        .attr("placeholder", "Registrierungsschlüssel")
}

fn pw_input() -> impl Element {
    RawHtmlEl::new("input")
        .attr("placeholder", "password")
        .attr("name", "password")
}

fn submit_button() -> impl Element {
    RawHtmlEl::new("button")
        .attr("type", "submit")
        .attr("class", "contrast")
        .child("Registrieren")
}
// ------ ------
//     Start
// ------ ------

#[wasm_bindgen(start)]
pub fn start() {
    start_app("app", page);
}
