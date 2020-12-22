use std::fmt::{ self, Display, Formatter };
use std::error::Error;
use rand::seq::SliceRandom;
use reqwest;
use serde::Deserialize;

pub fn generate(start_word: &str, search_type: SearchType) -> Result<Vec<String>, SearchError> {
    let request = match reqwest::blocking::get(&create_query(start_word, search_type)) {
        Ok(val) => val,
        Err(err) => return Err(SearchError::InternetError(Box::new(err)))
    };
    let word_list: Vec<WordResult> = match request.json() {
        Ok(val) => val,
        Err(err) => return Err(SearchError::OtherError(Box::new(err)))
    };
    let word_list_parsed = match parse_tags(word_list) {
        Ok(val) => val,
        Err(err) => return Err(SearchError::OtherError(Box::new(err)))
    };

    let mut word_list: Vec<String> = Vec::new();
    for word in word_list_parsed {
        if word.frequency < 300.0 && word.frequency != 0.0 && word.word.len() > 1 {
            word_list.push(word.word);
        }
    }

    let len = word_list.len();
    word_list[0..len].shuffle(&mut rand::thread_rng());

    if len < 8 {
        return Err(
            SearchError::MyError(
                format!("Not enough words could be found for \"{}\", please try a different word or category",
                start_word)
            )
        );
    }

    Ok(word_list)
}

fn create_query(word: &str, search_type: SearchType) -> String {
    let arg = match search_type {
        SearchType::RelatedTo => format!("rel_trg={}", word),
        SearchType::RhymesWith => format!("rel_rhy={}", word),
        SearchType::EndsWith => format!("sp=*{}", word),
        SearchType::SoundsLike => format!("sl={}", word),
        SearchType::BlankWord => format!("rel_bgb={}", word),
        SearchType::WordBlank => format!("rel_bga={}", word)
    };

    format!("https://api.datamuse.com/words?{}&md=f", arg)
}

fn parse_tags(word_list: Vec<WordResult>) -> Result<Vec<Word>, SearchError> {
    let mut word_list_parsed = Vec::new();
    for word_res in word_list {
        let word = word_res.word.clone();
        let mut freq: f64 = 0.0;
        if word_res.tags.len() != 0 {
            freq = match word_res.tags[0].replace("f:", "").parse() {
                Ok(val) => val,
                Err(err) => return Err(SearchError::OtherError(Box::new(err)))
            };
        }

        word_list_parsed.push(Word {
            word: word,
            frequency: freq
        });
    }

    Ok(word_list_parsed)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchType {
    RelatedTo,
    RhymesWith,
    EndsWith,
    SoundsLike,
    BlankWord,
    WordBlank
}

#[derive(Debug)]
pub enum SearchError<> {
    InternetError(Box<dyn Error>),
    OtherError(Box<dyn Error>),
    MyError(String)
}

#[derive(Deserialize)]
struct WordResult {
    word: String,
    _score: Option<u64>,
    tags: Vec<String>
}

struct Word {
    word: String,
    frequency: f64
}

impl Error for SearchError {}

impl Display for SearchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::InternetError(val) => write!(f, "{}", val),
            Self::OtherError(val) => write!(f, "{}", val),
            Self::MyError(val) => write!(f, "{}", val)
        }?;

        Ok(())
    }
}