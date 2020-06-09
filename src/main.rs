#[macro_use]
extern crate chan;

mod epd;

use epd::display::d7in5_v2::Display;
use epd::paint::Image;



fn main() {



      println!("e-Paper Init and Clear...");
      let mut display = Display::init();
      display.clear();
      epd::device::delay_ms(100);

      let display_tick = chan::tick_ms(Display::update_rate());



      loop {
            chan_select! {
                  display_tick.recv() => {
                        update(&mut display);
                  }
            }
      }
}

fn update(display: &mut Display) {
      draw_stuff(display);
}

fn draw_stuff(display: &mut Display) {
      println!("Paint_NewImage");
      let mut image = epd::paint::new_image(epd::display::d7in5_v2::WIDTH, epd::display::d7in5_v2::HEIGHT, epd::paint::Color::White);

//
//
//    println!("show window BMP-----------------");
//    epd::paint::select_image(&mut black_image);
//    epd::paint::clear(epd::paint::Color::White);
//    epd::bmp::read_bmp(String::from("./pic/100x100.bmp"), 10, 10);
//    epd::display::d7in5_v2::display(&mut black_image);
//    epd::device::delay_ms(2000);
//
//    println!("show BMP------------------------");
//    epd::paint::select_image(&mut black_image);
//    epd::paint::clear(epd::paint::Color::White);
//    epd::bmp::read_bmp(String::from("./pic/7in5_V2.bmp"), 0, 0);
//    epd::display::d7in5_v2::display(&mut black_image);
//    epd::device::delay_ms(2000);
//
//
//    println!("show image for array -----------------");
//    epd::paint::select_image(&mut black_image);
//    epd::paint::clear(epd::paint::Color::White);
//    epd::paint::draw_bitmap(gImage_7in5_V2().into_boxed_slice());
//    epd::display::d7in5_v2::display(&mut black_image);
//    epd::device::delay_ms(2000);
//

      //println!("SelectImage:BlackImage");
      //epd::paint::select_image(&mut black_image);
      image.clear(epd::paint::Color::White);

      // 2.Drawing on the image
      //println!("Drawing:BlackImage");
      image.draw_point(10, 80, epd::paint::Color::Black, epd::paint::Dot_Pixel::DOT_PIXEL_1X1, epd::paint::Dot_Style::DOT_FILL_AROUND);
      image.draw_point(10, 90, epd::paint::Color::Black, epd::paint::Dot_Pixel::DOT_PIXEL_2X2, epd::paint::Dot_Style::DOT_FILL_AROUND);
      image.draw_point(10, 100, epd::paint::Color::Black, epd::paint::Dot_Pixel::DOT_PIXEL_3X3, epd::paint::Dot_Style::DOT_FILL_AROUND);

      image.draw_line(20, 70, 70, 120, epd::paint::Color::Black, epd::paint::Dot_Pixel::DOT_PIXEL_1X1, epd::paint::Line_Style::LINE_STYLE_SOLID);
      image.draw_line(70, 70, 20, 120, epd::paint::Color::Black, epd::paint::Dot_Pixel::DOT_PIXEL_1X1, epd::paint::Line_Style::LINE_STYLE_SOLID);

      image.draw_rectangle(20, 70, 70, 120, epd::paint::Color::Black, epd::paint::Dot_Pixel::DOT_PIXEL_1X1, epd::paint::Draw_Fill::DRAW_FILL_EMPTY);
      image.draw_rectangle(80, 70, 130, 120, epd::paint::Color::Black, epd::paint::Dot_Pixel::DOT_PIXEL_1X1, epd::paint::Draw_Fill::DRAW_FILL_FULL);
      image.draw_circle(45, 95, 20, epd::paint::Color::Black, epd::paint::Dot_Pixel::DOT_PIXEL_1X1, epd::paint::Draw_Fill::DRAW_FILL_EMPTY);
      image.draw_circle(105, 95, 20, epd::paint::Color::White, epd::paint::Dot_Pixel::DOT_PIXEL_1X1, epd::paint::Draw_Fill::DRAW_FILL_FULL);
      image.draw_line(85, 95, 125, 95, epd::paint::Color::Black, epd::paint::Dot_Pixel::DOT_PIXEL_1X1, epd::paint::Line_Style::LINE_STYLE_DOTTED);
      image.draw_line(105, 75, 105, 115, epd::paint::Color::Black, epd::paint::Dot_Pixel::DOT_PIXEL_1X1, epd::paint::Line_Style::LINE_STYLE_DOTTED);
      image.draw_string(10, 0, "waveshare", epd::paint::font16(), epd::paint::Color::Black, epd::paint::Color::White);
      image.draw_string(10, 20, "hello world", epd::paint::font12(), epd::paint::Color::White, epd::paint::Color::Black);
      image.draw_num(10, 33, 123456789, epd::paint::font12(), epd::paint::Color::Black, epd::paint::Color::White);
      image.draw_num(10, 50, 987654321, epd::paint::font16(), epd::paint::Color::White, epd::paint::Color::Black);

      display.display(image);

      //println!("EPD_Display");

      //epd::device::delay_ms(5000);
      //
      // println!("Clear...");
      // //epd::display::d7in5_v2::clear();
      //
      // println!("Goto Sleep...");
      // epd::display::d7in5_v2::sleep();
      //
      // // close 5V
      // println!("close 5V, Module enters 0 power consumption ...");
      // epd::device::module_exit()
}
