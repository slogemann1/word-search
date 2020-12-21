use std::io::Error;
use std::sync::Mutex;
use std::thread;
use pdf_canvas::{ Pdf, BuiltinFont, FontSource, Canvas };
use pdf_canvas::graphicsstate::Color;

use crate::request::TitledWordSearch;

static FONT: BuiltinFont = BuiltinFont::Times_Roman;

lazy_static! {
    static ref COUNT: Mutex<u32> = Mutex::new(0);
    static ref MAX_COUNT: Mutex<u32> = Mutex::new(0); 
}

pub fn create_pdf(word_search_list: Vec<TitledWordSearch>, width: f32, height: f32, directory: &str) -> Result<(), Error> {
    let mut pdf = Pdf::create(&format!("{}/wordsearch.pdf", directory))?;

    set_max_count(word_search_list.len() as u32);
    set_count(0);

    for word_search in word_search_list {
        set_count(get_count() + 1);
        pdf.render_page(width, height, |mut canvas| {
            draw_page(&word_search, &mut canvas, width, height)?;
            Ok(())
        })?;
    }

    pdf.finish()?;

    set_count(0);
    set_max_count(1);

    Ok(())
}

pub fn get_progress() -> f32 {
    (get_count() as f32) / (get_max_count() as f32)
}

fn draw_page(word_search: &TitledWordSearch, canvas: &mut Canvas, width: f32, height: f32) -> Result<(), Error> {
    let mut title: Vec<char> = word_search.title.chars().collect();
    title[0] = title[0].to_string().to_uppercase().chars().next().unwrap();
    let title: String = title.into_iter().collect();
    canvas.center_text(width / 2.0, height - height / 20.0, FONT, get_font_size(width) * 1.5, &title)?;

    let w = word_search.word_search.field.len() as f32;
    let h = word_search.word_search.field[0].len() as f32;
    let space = (width - width / 4.0) / w;
    draw_field(canvas, &word_search.word_search.field, w, h, space, width, height)?;

    let word_list_start = get_start_search(height) - h * space - height / 32.0;
    draw_word_list(canvas, &word_search.word_search.word_list, word_list_start, width, height)?;

    Ok(())
}

fn draw_field(canvas: &mut Canvas, word_search: &Vec<Vec<char>>, w: f32, h: f32, space: f32, width: f32, height: f32) -> Result<(), Error> {
    for x in 0..(w as usize) {
        for y in 0..(h as usize) {
            let x_pos = (x as f32) * space + width / 8.0 + space / 2.0;
            let y_pos = get_start_search(height) - (y as f32) * space;
            print_char(canvas, x_pos, y_pos, word_search[x][y], width)?;
        }
    }

    canvas.set_stroke_color(Color::rgb(0, 0, 0))?;
    canvas.rectangle(width / 8.0 - space / 8.0, get_start_search(height) - space * h + space / 2.0, space * w + space / 4.0, space * h + space / 4.0)?;
    canvas.stroke()?;

    Ok(())
}

fn draw_word_list(canvas: &mut Canvas, word_list: &Vec<String>, start_y: f32, width: f32, height: f32) -> Result<(), Error> {
    let mut max_width: f32 = 0.0;
    for word in word_list {
        let width = FONT.get_width(get_font_size(width) * 0.75, &word);
        if width > max_width {
            max_width = width;
        }
    }
    let cols = ((width - width / 4.0) / (max_width * 1.25)) as usize;
    let space_x = (width - width / 4.0) / (cols as f32);

    let mut rows = word_list.len() / cols;
    if word_list.len() % cols != 0 {
        rows += 1;
    }
    let tot_height = start_y - height / 32.0;
    let space_y = tot_height / (rows as f32);

    for x in 0..cols {
        for y in 0..rows {
            if x + y * cols >= word_list.len() {
                break;
            }

            let x_pos = height / 8.0 + (x as f32) * space_x;
            let y_pos = start_y - (y as f32) * space_y;

            canvas.left_text(x_pos, y_pos, FONT, get_font_size(width) * 0.75, &word_list[x + y * cols])?;
        }
    }

    Ok(())
}

fn print_char(canvas: &mut Canvas, x: f32, y: f32, c: char, width: f32) -> Result<(), Error> {
    canvas.center_text(x, y, FONT, get_font_size(width), &c.to_string())?;
    Ok(())
}

fn get_font_size(width: f32) -> f32 {
    width / 30.0
}

fn get_start_search(height: f32) -> f32 {
    height - height / 7.0
}

fn set_count(count: u32) {
    match COUNT.lock() {
        Ok(mut val) => *val = count,
        Err(_) => {
            thread::sleep(std::time::Duration::from_millis(50));
            set_count(count);
        }
    }
}

fn get_count() -> u32 {
    match COUNT.lock() {
        Ok(val) => *val,
        Err(_) => {
            thread::sleep(std::time::Duration::from_millis(50));
            get_count()
        }
    }
}

fn set_max_count(max_count: u32) {
    match MAX_COUNT.lock() {
        Ok(mut val) => *val = max_count,
        Err(_) => {
            thread::sleep(std::time::Duration::from_millis(50));
            set_max_count(max_count);
        }
    }
}

fn get_max_count() -> u32 {
    match MAX_COUNT.lock() {
        Ok(val) => *val,
        Err(_) => {
            thread::sleep(std::time::Duration::from_millis(50));
            get_max_count()
        }
    }
}