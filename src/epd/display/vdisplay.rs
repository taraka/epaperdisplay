extern crate orbclient;

use orbclient::{Color, EventOption, GraphicsPath, Mode, Renderer, Window};
use crate::epd::paint::Image;

pub const WIDTH: u16 = 800;
pub const HEIGHT: u16 = 480;


pub struct Display {
    window: Window
}

impl Display {
    pub fn init() -> Display  {
        let mut display = Display {
            window: Window::new(
                WIDTH as i32,
                HEIGHT as i32,
                WIDTH as u32,
                HEIGHT as u32,
                "TITLE",
            ).unwrap(),
        };

        return display;
    }


    pub fn clear(&mut self) {}

    pub fn clear_black(&mut self) {}

    pub fn display(&mut self, image: Image) {

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                //println!("x: {:<5}, y: {:<5}, data: {}", x, y, (x + y * (WIDTH/8)));
                let data = !image.image[((x / 8) + y * (WIDTH / 8)) as usize];
                if (0x80 >> (x % 8)) & data != 0 {
                    self.window.pixel(x as i32, y as i32, Color::rgb(0, 0, 0));
                } else {
                    self.window.pixel(x as i32, y as i32, Color::rgb(255, 255, 255));
                }
            }

        }

        self.window.sync();


        for event in self.window.events() {
            match event.to_option() {
                EventOption::Quit(_quit_event) => std::process::exit(0),
                _ => {},
            }
        }
    }

    pub fn update_rate() -> u32 {
        100
    }
}

pub fn reset()
{

}

pub fn sleep() {

}