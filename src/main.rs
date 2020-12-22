#![windows_subsystem = "windows"]

extern crate rand;
extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate pdf_canvas;
extern crate iced;
extern crate imagine;

#[macro_use]
extern crate lazy_static;

mod word_search;
mod word_list;
mod request;
mod pdf;
mod gui;
mod config;
mod img;

fn main() {
    match gui::run() {
        Ok(_) => (),
        Err(_) => ()
    };
}