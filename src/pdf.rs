use std::io::Error;
use pdf_canvas::{ Pdf, BuiltinFont, FontSource, Canvas };
use pdf_canvas::graphicsstate::Color;

use crate::request::TitledWordSearch;

static WIDTH: f32 = 612.0;
static HEIGHT: f32 = 792.0;
static START_SEARCH: f32 = HEIGHT - HEIGHT / 7.0;
static FONT: BuiltinFont = BuiltinFont::Times_Roman;
static FONT_SIZE: f32 = WIDTH / 30.0;

pub fn create_pdf(word_search_list: Vec<TitledWordSearch>) -> Result<(), Error> {
    let mut pdf = Pdf::create("test.pdf")?;

    for word_search in word_search_list {
        pdf.render_page(WIDTH, HEIGHT, |mut canvas| {
            draw_page(&word_search, &mut canvas)?;
            Ok(())
        })?;
    }

    pdf.finish()?;

    Ok(())
}

fn draw_page(word_search: &TitledWordSearch, canvas: &mut Canvas) -> Result<(), Error> {
    let mut title: Vec<char> = word_search.title.chars().collect();
    title[0] = title[0].to_string().to_uppercase().chars().next().unwrap();
    let title: String = title.into_iter().collect();
    canvas.center_text(WIDTH / 2.0, HEIGHT - HEIGHT / 20.0, FONT, FONT_SIZE * 1.5, &title)?;

    let w = word_search.word_search.field.len() as f32;
    let h = word_search.word_search.field[0].len() as f32;
    let space = (WIDTH - WIDTH / 4.0) / w;
    draw_field(canvas, &word_search.word_search.field, w, h, space)?;

    let word_list_start = START_SEARCH - h * space - HEIGHT / 32.0;
    draw_word_list(canvas, &word_search.word_search.word_list, word_list_start)?;

    Ok(())
}

fn draw_field(canvas: &mut Canvas, word_search: &Vec<Vec<char>>, w: f32, h: f32, space: f32) -> Result<(), Error> {
    for x in 0..(w as usize) {
        for y in 0..(h as usize) {
            let x_pos = (x as f32) * space + WIDTH / 8.0 + space / 2.0;
            let y_pos = START_SEARCH - (y as f32) * space;
            print_char(canvas, x_pos, y_pos, word_search[x][y])?;
        }
    }

    canvas.set_stroke_color(Color::rgb(0, 0, 0))?;
    canvas.rectangle(WIDTH / 8.0 - space / 8.0, START_SEARCH - space * h + space / 2.0, space * w + space / 4.0, space * h + space / 4.0)?;
    canvas.stroke()?;

    Ok(())
}

fn draw_word_list(canvas: &mut Canvas, word_list: &Vec<String>, start_y: f32) -> Result<(), Error> {
    let mut max_width: f32 = 0.0;
    for word in word_list {
        let width = FONT.get_width(FONT_SIZE * 0.75, &word);
        if width > max_width {
            max_width = width;
        }
    }
    let cols = ((WIDTH - WIDTH / 4.0) / (max_width * 1.25)) as usize;
    let space_x = (WIDTH - WIDTH / 4.0) / (cols as f32);

    let mut rows = word_list.len() / cols;
    if word_list.len() % cols != 0 {
        rows += 1;
    }
    let tot_height = start_y - HEIGHT / 32.0;
    let space_y = tot_height / (rows as f32);

    for x in 0..cols {
        for y in 0..rows {
            if x + y * cols >= word_list.len() {
                break;
            }

            let x_pos = HEIGHT / 8.0 + (x as f32) * space_x;
            let y_pos = start_y - (y as f32) * space_y;

            canvas.left_text(x_pos, y_pos, FONT, FONT_SIZE * 0.75, &word_list[x + y * cols])?;
        }
    }

    Ok(())
}

fn print_char(canvas: &mut Canvas, x: f32, y: f32, c: char) -> Result<(), Error> {
    canvas.center_text(x, y, FONT, FONT_SIZE, &c.to_string())?;
    Ok(())
}
