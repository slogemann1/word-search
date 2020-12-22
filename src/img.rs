use std::fs;
use imagine::png;
use iced::window::icon::Icon;

pub fn get_icon() -> Option<Icon> {
    let (data, width, height) = match load_image_bytes() {
        Some(val) => val,
        None => return None
    };
    match Icon::from_rgba(data, width as u32, height as u32) {
        Ok(val) => Some(val),
        Err(err) => {
            println!("err: {}", err);
            None
        }
    } 
}

fn load_image_bytes() -> Option<(Vec<u8>, usize, usize)> {
    let read_bytes = match fs::read("./data/icon.png") {
        Ok(val) => val,
        Err(_) => return None
    };
    let png = match png::parse_png_rgba8(read_bytes.as_slice()) {
        Ok(val) => val,
        Err(err) => {
            println!("Error: {:#?}", err);
            return None;
        }
    };
    let data = png.bitmap.pixels();
    
    let mut new_data: Vec<u8> = Vec::new();
    for pix in data {
        new_data.push(pix.r);
        new_data.push(pix.g);
        new_data.push(pix.b);
        new_data.push(pix.a);
    }

    Some(
        (new_data, png.bitmap.width(), png.bitmap.height())
    )
}