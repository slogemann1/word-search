use std::sync::Mutex;
use std::thread;

use crate::word_list::{ self, SearchType, SearchError };
use crate::word_search::{ self, WordSearch };

lazy_static! {
    pub static ref TOTAL: Mutex<i32> = {
        Mutex::new(0)
    };

    pub static ref FINISHED: Mutex<bool> = {
        Mutex::new(false)
    };
}

pub fn handle_requests(requests: Vec<WordSearchRequest>) -> (Vec<TitledWordSearch>, Vec<String>) {
    set_finished(false);
    calc_total(&requests);
    let mut error_msgs: Vec<String> = Vec::new();
    let mut word_search_list: Vec<TitledWordSearch> = Vec::new();
    for request in requests {
        let word_search = match get_word_search(&request.word, request.stype.clone(), request.max_count, request.width, request.height) {
            Ok(val) => val,
            Err(err) => {
                error_msgs.push(format!("{}", err));
                continue;
            }
        };
        let title = match request.stype {
            SearchType::RelatedTo => format!("{}", request.word),
            SearchType::RhymesWith => format!("Rhymes with {}", request.word),
            SearchType::EndsWith => format!("Ends with -{}", request.word),
            SearchType::SoundsLike => format!("Sounds like {}", request.word),
            SearchType::BlankWord => format!("____ {}", request.word),
            SearchType::WordBlank => format!("{} ____", request.word)
        };

        word_search_list.push(TitledWordSearch {
            title: title,
            word_search: word_search
        });
    }

    word_search::set_count(0);
    set_total(1);
    set_finished(true);

    (word_search_list, error_msgs)
}

pub fn get_progress() -> f32 {
    (word_search::get_count() as f32) / (get_total() as f32)
}

pub fn get_finished() -> bool {
    match FINISHED.lock() {
        Ok(val) => *val,
        Err(_) => {
            thread::sleep(std::time::Duration::from_millis(50));
            get_finished()
        }
    }
}

fn get_word_search(start_word: &str, search_type: SearchType, max_count: usize, width: usize, height: usize) -> Result<WordSearch, SearchError> {
    let word_list = match word_list::generate(start_word, search_type) {
        Ok(val) => val,
        Err(val) => return Err(val)
    };

    let mut word_search_list: Vec<WordSearch> = Vec::new();
    for _ in 0..10 {
        let word_search = match word_search::generate(&word_list, max_count, width, height) {
            Some(val) => val,
            None => {
                continue;
            }
        };

        word_search_list.push(word_search);
    }

    if word_search_list.len() == 0 {
        return Err(
            SearchError::MyError(
                String::from("The requested word search could not be generated (try increasing the field size)")
            )
        );
    }

    let mut final_result = word_search_list[0].clone();
    for word_search in word_search_list {
        if word_search.word_list.len() > final_result.word_list.len() {
            final_result = word_search.clone();
        }
    }

    Ok(final_result)
}

fn calc_total(requests: &Vec<WordSearchRequest>) {
    let mut tot_words = 0;
    for request in requests {
        tot_words += request.max_count;
    }

    set_total(tot_words as i32);
}

fn set_total(total: i32) {
    match TOTAL.lock() {
        Ok(mut val) => *val = total,
        Err(_) => {
            thread::sleep(std::time::Duration::from_millis(50));
            set_total(total);
        }
    }
}

fn get_total() -> i32 {
    match TOTAL.lock() {
        Ok(val) => *val,
        Err(_) => {
            thread::sleep(std::time::Duration::from_millis(50));
            get_total()
        }
    }
}

fn set_finished(finished: bool) {
    match FINISHED.lock() {
        Ok(mut val) => *val = finished,
        Err(_) => {
            thread::sleep(std::time::Duration::from_millis(50));
            set_finished(finished);
        }
    }
}

pub struct WordSearchRequest {
    pub word: String,
    pub stype: SearchType,
    pub max_count: usize,
    pub height: usize,
    pub width: usize 
}

pub struct TitledWordSearch {
    pub title: String,
    pub word_search: WordSearch
}