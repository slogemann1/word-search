extern crate rand;
extern crate reqwest;
extern crate serde;
extern crate pdf_canvas;
extern crate iced;

#[macro_use]
extern crate lazy_static;

mod word_search;
mod word_list;
mod request;
mod pdf;
mod gui;

fn main() {
    match gui::run() {
        Ok(_) => (),
        Err(err) => println!("{}", err)
    };
}