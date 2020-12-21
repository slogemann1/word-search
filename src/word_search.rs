use std::sync::Mutex;
use std::thread;
use std::iter::FromIterator;
use rand::prelude::*;

static MAX_ITER: u32 = 1000;

lazy_static! {
    static ref ITERATIONS: Mutex<u32> = {
        Mutex::new(0)
    };

    pub static ref COUNT: Mutex<u32> = { //For getting progress
        Mutex::new(0)
    };
}

pub fn generate(word_list: &Vec<String>, max_count: usize, height: usize, width: usize) -> Option<WordSearch> {
    let mut field: Vec<Vec<char>> = new_field(height, width);
    let mut random = rand::thread_rng();
    let word_list = word_list.clone();
    let word_list: Vec<String> = word_list.into_iter().filter(|word| word.len() <= width && word.len() <= height).collect();

    let mut search_list: Vec<String> = Vec::new();

    for word in word_list {
        if check_stop() {
            return None;
        }
        else if search_list.len() >= max_count {
            break;
        }

        let word_original = word.clone();
        let mut word = word.to_uppercase().replace(" ", "");
        if random.gen::<f32>() < 0.25 {
            word = String::from_iter(word.chars().rev());
        }

        match add_word(&field, &word, &mut random) {
            Some(val) => {
                set_count(get_count() + 1);
                field = val;
                search_list.push(word_original.to_uppercase());
            },
            None => ()
        }
    }

    for x in 0..field.len() {
        for y in 0..field[0].len() {
            if field[x][y] == (0 as char) {
                field[x][y] = (((random.gen::<f32>() * 26.0) as u8) + 65) as char;
            }
        }
    }

    set_iter(0);

    Some(
        WordSearch {
            field: field,
            word_list: search_list
        }
    )
}

pub fn get_count() -> u32 {
    match COUNT.lock() {
        Ok(val) => *val,
        Err(_) => {
            thread::sleep(std::time::Duration::from_millis(50));
            get_count()
        }
    }
}

fn check_stop() -> bool {
    match ITERATIONS.lock() {
        Ok(val) => {
            if *val > MAX_ITER {
                return true;
            }
            false
        },
        Err(_) => false
    }
}

fn set_iter(iter: u32) {
    match ITERATIONS.lock() {
        Ok(mut val) => *val = iter,
        Err(_) => {
            thread::sleep(std::time::Duration::from_millis(50));
            set_iter(iter);
        }
    }
}

fn get_iter() -> u32 {
    match ITERATIONS.lock() {
        Ok(val) => *val,
        Err(_) => {
            thread::sleep(std::time::Duration::from_millis(50));
            get_iter()
        }
    }
}

pub fn set_count(count: u32) {
    match COUNT.lock() {
        Ok(mut val) => *val = count,
        Err(_) => {
            thread::sleep(std::time::Duration::from_millis(50));
            set_count(count)
        }
    }
}

fn add_word(field: &Vec<Vec<char>>, word: &str, random: &mut ThreadRng) -> Option<Vec<Vec<char>>> {
    let start_or = (random.gen::<f32>() * 4.0) as usize; //Orientations: horizontal (0), vertical (1), diagonal down (2), diagonal up (3),
    let or_set = vec![
        start_or,
        start_or + 1 % 4,
        start_or + 2 % 4,
        start_or + 3 % 4
    ];
    let start_w = (random.gen::<f32>() * (field.len() as f32)) as usize;
    let start_h = (random.gen::<f32>() * (field[0].len() as f32)) as usize;

    set_iter(get_iter() + 1);

    for orientation in or_set {
        for x in start_w..field.len() {
            for y in start_h..field[0].len() {
                if check_stop() {
                    return None;
                }

                match try_add(field, word, x, y, orientation) {
                    Some(val) => {
                        return Some(val)
                    },
                    None => continue
                }
            }
        }

        for x in 0..start_w {
            for y in 0..start_h {
                if check_stop() {
                    return None;
                }

                match try_add(field, word, x, y, orientation) {
                    Some(val) => {
                        return Some(val)
                    },
                    None => continue
                }
            }
        }
    }

    None
}

fn try_add(field: &Vec<Vec<char>>, word: &str, x: usize, y: usize, orientation: usize) -> Option<Vec<Vec<char>>> {
    let word: Vec<char> = word.chars().collect();
    let w = field.len();
    let h = field[0].len();
    
    let mut field_copy = field.clone();

    match orientation {
        0 => {
            if x + word.len() >= w {
                return None;
            }

            for i in 0..word.len() {
                if field[x+i][y] != (0 as char) && field[x+i][y] != word[i] {
                    return None;
                }
                field_copy[x+i][y] = word[i];
            }
        },
        1 => {
            if y + word.len() >= h {
                return None;
            }

            for i in 0..word.len() {
                if field[x][y+i] != (0 as char) && field[x][y+i] != word[i] {
                    return None;
                }
                field_copy[x][y+i] = word[i];
            }
        },
        2 => {
            if x + word.len() >= w || y + word.len() >= h {
                return None;
            }

            for i in 0..word.len() {
                if field[x+i][y+i] != (0 as char) && field[x+i][y+i] != word[i] {
                    return None;
                }
                field_copy[x+i][y+i] = word[i];
            }
        },
        3 => {
            if x + word.len() >= w || (y as i32) - (word.len() as i32) < 0 {
                return None;
            }

            for i in 0..word.len() {
                let y_index = ((y as i32)-(i as i32)) as usize;
                if field[x+i][y_index] != (0 as char) && field[x+i][y_index] != word[i] {
                    return None;
                }
                field_copy[x+i][y_index] = word[i];
            }
        },
        _ => return None
    }

    Some(field_copy)
}

fn new_field(width: usize, height: usize) -> Vec<Vec<char>> {
    let mut field: Vec<Vec<char>> = Vec::new();

    for x in 0..width {
        field.push(Vec::new());
        for _ in 0..height {
            field[x].push(0 as char); 
        }
    }

    field
}

#[derive(Clone)]
pub struct WordSearch {
    pub field: Vec<Vec<char>>,
    pub word_list: Vec<String>
}