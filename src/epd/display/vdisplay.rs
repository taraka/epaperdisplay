extern crate orbclient;

use orbclient::{Color, EventOption, GraphicsPath, Mode, Renderer, Window};
use crate::epd::paint::Image;

pub const WIDTH: u16 = 800;
pub const HEIGHT: u16 = 480;

static mut window: Option<Window> = None;

pub fn init() {
    unsafe {
        window = Some(Window::new(
            WIDTH as i32,
            HEIGHT as i32,
            WIDTH as u32,
            HEIGHT as u32,
            "TITLE",
        ).unwrap());
    }
}

pub fn clear() {

}

pub fn clear_black() {

}

pub fn display(image: &Image) {
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            //println!("x: {:<5}, y: {:<5}, data: {}", x, y, (x + y * (WIDTH/8)));
            let data = !image.image[((x/8) + y * (WIDTH/8)) as usize];
            if  (0x80 >> (x % 8)) & data != 0 {
                unsafe { window.as_mut().unwrap() }.pixel(x as i32, y as i32, Color::rgb(0, 0, 0));
            }
            else {
                unsafe { window.as_mut().unwrap() }.pixel(x as i32, y as i32, Color::rgb(255, 255, 255));
            }

        }
    }

    unsafe { window.as_mut().unwrap() }.sync();
}

pub fn reset()
{

}

pub fn sleep() {

}