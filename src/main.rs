extern crate rand;
extern crate reqwest;
extern crate serde;
extern crate pdf_canvas;

#[macro_use]
extern crate lazy_static;

mod word_search;
mod word_list;
mod request;
mod pdf;

use word_list::SearchType;
use request::{ WordSearchRequest };

fn main() {
    let requests = get_user_requests();

    let (word_searches, errors) = request::handle_requests(requests);
    match pdf::create_pdf(word_searches) {
        Ok(_) => (),
        Err(err) => println!("Error: Could not create pdf\nError Message: {}", err)
    }
}

fn get_user_requests() -> Vec<WordSearchRequest> {
    return vec!(
        WordSearchRequest {
            word: String::from("on"),
            stype: SearchType::EndsWith,
            max_count: 20,
            height: 12,
            width: 12
        },
        WordSearchRequest {
            word: String::from("soda"),
            stype: SearchType::WordBlank,
            max_count: 20,
            height: 12,
            width: 12
        },
    );
}