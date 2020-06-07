mod epd;

fn main() {
   draw_stuff();
}


fn draw_stuff() {
      println!("EPD_7IN5_V2_test Demo");

      epd::device::module_init().expect("Fail to init device with code: {}");

      println!("e-Paper Init and Clear...");
      epd::display::d7in5_v2::init();
      epd::display::d7in5_v2::clear();
      epd::device::delay_ms(500);


      println!("Paint_NewImage");
      let mut black_image = epd::paint::new_image(epd::display::d7in5_v2::WIDTH, epd::display::d7in5_v2::HEIGHT);

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

      println!("SelectImage:BlackImage");
      epd::paint::select_image(&mut black_image);
      epd::paint::clear(epd::paint::Color::White);

      // 2.Drawing on the image
      println!("Drawing:BlackImage");
      epd::paint::draw_point(10, 80, epd::paint::Color::Black, epd::paint::Dot_Pixel::DOT_PIXEL_1X1, epd::paint::Dot_Style::DOT_FILL_AROUND);
      epd::paint::draw_point(10, 90, epd::paint::Color::Black, epd::paint::Dot_Pixel::DOT_PIXEL_2X2, epd::paint::Dot_Style::DOT_FILL_AROUND);
      epd::paint::draw_point(10, 100, epd::paint::Color::Black, epd::paint::Dot_Pixel::DOT_PIXEL_3X3, epd::paint::Dot_Style::DOT_FILL_AROUND);

      epd::paint::draw_line(20, 70, 70, 120, epd::paint::Color::Black, epd::paint::Dot_Pixel::DOT_PIXEL_1X1, epd::paint::Line_Style::LINE_STYLE_SOLID);
      epd::paint::draw_line(70, 70, 20, 120, epd::paint::Color::Black, epd::paint::Dot_Pixel::DOT_PIXEL_1X1, epd::paint::Line_Style::LINE_STYLE_SOLID);

      epd::paint::draw_rectangle(20, 70, 70, 120, epd::paint::Color::Black, epd::paint::Dot_Pixel::DOT_PIXEL_1X1, epd::paint::Draw_Fill::DRAW_FILL_EMPTY);
      epd::paint::draw_rectangle(80, 70, 130, 120, epd::paint::Color::Black, epd::paint::Dot_Pixel::DOT_PIXEL_1X1, epd::paint::Draw_Fill::DRAW_FILL_FULL);
      epd::paint::draw_circle(45, 95, 20, epd::paint::Color::Black, epd::paint::Dot_Pixel::DOT_PIXEL_1X1, epd::paint::Draw_Fill::DRAW_FILL_EMPTY);
      epd::paint::draw_circle(105, 95, 20, epd::paint::Color::White, epd::paint::Dot_Pixel::DOT_PIXEL_1X1, epd::paint::Draw_Fill::DRAW_FILL_FULL);
      epd::paint::draw_line(85, 95, 125, 95, epd::paint::Color::Black, epd::paint::Dot_Pixel::DOT_PIXEL_1X1, epd::paint::Line_Style::LINE_STYLE_DOTTED);
      epd::paint::draw_line(105, 75, 105, 115, epd::paint::Color::Black, epd::paint::Dot_Pixel::DOT_PIXEL_1X1, epd::paint::Line_Style::LINE_STYLE_DOTTED);
      epd::paint::draw_string(10, 0, String::from("waveshare"), epd::paint::font16(), epd::paint::Color::Black, epd::paint::Color::White);
      epd::paint::draw_string(10, 20, String::from("hello world"), epd::paint::font12(), epd::paint::Color::White, epd::paint::Color::Black);
      epd::paint::draw_num(10, 33, 123456789, epd::paint::font12(), epd::paint::Color::Black, epd::paint::Color::White);
      epd::paint::draw_num(10, 50, 987654321, epd::paint::font16(), epd::paint::Color::White, epd::paint::Color::Black);

      println!("EPD_Display");
      epd::display::d7in5_v2::display(&mut black_image);
      epd::device::delay_ms(2000);

      println!("Clear...");
      //epd::display::d7in5_v2::clear();

      println!("Goto Sleep...");
      epd::display::d7in5_v2::sleep();

      // close 5V
      println!("close 5V, Module enters 0 power consumption ...");
      epd::device::module_exit()
}
