#[macro_use]
extern crate chan;

mod epd;




extern crate hyper;
extern crate hyper_rustls;
extern crate yup_oauth2 as oauth2;
extern crate google_calendar3 as calendar3;
use calendar3::Channel;
use calendar3::{Result, Error};
use std::default::Default;
use oauth2::{Authenticator, DefaultAuthenticatorDelegate, ApplicationSecret, MemoryStorage};
use calendar3::CalendarHub;



fn main() {
      init_display();
      update_data();

      let tick = chan::tick_ms(5000);
      loop {
            chan_select! {
                  tick.recv() => {
                        update()
                  }
              }
      }

}

fn update() {
      update_data();
      draw_stuff();
}

fn update_data() {
// Get an ApplicationSecret instance by some means. It contains the `client_id` and
// `client_secret`, among other things.
      let secret: ApplicationSecret = Default::default();
// Instantiate the authenticator. It will choose a suitable authentication flow for you,
// unless you replace  `None` with the desired Flow.
// Provide your own `AuthenticatorDelegate` to adjust the way it operates and get feedback about
// what's going on. You probably want to bring in your own `TokenStorage` to persist tokens and
// retrieve them from storage.
      let auth = Authenticator::new(&secret, DefaultAuthenticatorDelegate,
                                    hyper::Client::with_connector(hyper::net::HttpsConnector::new(hyper_rustls::TlsClient::new())),
                                    <MemoryStorage as Default>::default(), None);
      let mut hub = CalendarHub::new(hyper::Client::with_connector(hyper::net::HttpsConnector::new(hyper_rustls::TlsClient::new())), auth);
// As the method needs a request, you would usually fill it with the desired information
// into the respective structure. Some of the parts shown here might not be applicable !
// Values shown here are possibly random and not representative !
      let mut req = Channel::default();

// You can configure optional parameters by calling the respective setters at will, and
// execute the final call using `doit()`.
// Values shown here are possibly random and not representative !
      let result = hub.events().watch(req, "calendarId")
          .updated_min("ea")
          .time_zone("no")
          .time_min("justo")
          .time_max("justo")
          .sync_token("et")
          .single_events(true)
          .show_hidden_invitations(true)
          .show_deleted(false)
          .add_shared_extended_property("Lorem")
          .q("et")
          .add_private_extended_property("duo")
          .page_token("aliquyam")
          .order_by("sea")
          .max_results(-55)
          .max_attendees(-75)
          .i_cal_uid("erat")
          .always_include_email(false)
          .doit();

      match result {
            Err(e) => match e {
                  // The Error enum provides details about what exactly happened.
                  // You can also just use its `Debug`, `Display` or `Error` traits
                  Error::HttpError(_)
                  |Error::MissingAPIKey
                  |Error::MissingToken(_)
                  |Error::Cancelled
                  |Error::UploadSizeLimitExceeded(_, _)
                  |Error::Failure(_)
                  |Error::BadRequest(_)
                  |Error::FieldClash(_)
                  |Error::JsonDecodeError(_, _) => println!("{}", e),
            },
            Ok(res) => println!("Success: {:?}", res),
      }
}

fn init_display() {
      println!("EPD_7IN5_V2_test Demo");

      epd::device::module_init().expect("Fail to init device with code: {}");

      println!("e-Paper Init and Clear...");
      epd::display::d7in5_v2::init();
      epd::display::d7in5_v2::clear();
      epd::device::delay_ms(500);
}

fn draw_stuff() {
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

      println!("SelectImage:BlackImage");
      //epd::paint::select_image(&mut black_image);
      image.clear(epd::paint::Color::White);

      // 2.Drawing on the image
      println!("Drawing:BlackImage");
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

      println!("EPD_Display");
      epd::display::d7in5_v2::display(&image);
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
