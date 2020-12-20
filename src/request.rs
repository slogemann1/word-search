use crate::word_list::{ self, SearchType, SearchError };
use crate::word_search::{ self, WordSearch };

pub fn handle_requests(requests: Vec<WordSearchRequest>) -> (Vec<TitledWordSearch>, Vec<String>) {
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

    (word_search_list, error_msgs)
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