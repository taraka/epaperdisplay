extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;
use sdl2::rect::Point;

use crate::epd::paint::Image;

pub const WIDTH: u16 = 800;
pub const HEIGHT: u16 = 480;


pub struct Display {
    canvas: sdl2::render::WindowCanvas,
    event_pump: sdl2::EventPump
}

impl Display {
    pub fn init() -> Display  {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem.window("rust-sdl2 demo", 800, 480)
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();

        Display {
            canvas,
            event_pump: sdl_context.event_pump().unwrap()
        }
    }


    pub fn clear(&mut self) {}

    pub fn clear_black(&mut self) {}

    pub fn display(&mut self, image: Image) {

        let canvas = &mut self.canvas;

        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas.clear();


        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                //println!("x: {:<5}, y: {:<5}, data: {}", x, y, (x + y * (WIDTH/8)));
                let data = !image.image[((x / 8) + y * (WIDTH / 8)) as usize];
                if (0x80 >> (x % 8)) & data != 0 {
                    canvas.set_draw_color(Color::RGB(0, 0, 0));
                } else {
                    canvas.set_draw_color(Color::RGB(255, 255, 255));
                }
                canvas.draw_point(Point::new(x as i32, y as i32));
            }
        }

        canvas.present();

        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    std::process::exit(0);
                },
                _ => {}
            }
        }

        // self.window.sync();
        //
        //
        // for event in self.window.events() {
        //     match event.to_option() {
        //         EventOption::Quit(_quit_event) => std::process::exit(0),
        //         _ => {},
        //     }
        // }
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