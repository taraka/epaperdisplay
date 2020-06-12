#[macro_use]
extern crate chan;
mod epd;
use epd::display::d7in5_v2::Display;
use epd::paint::Image;
use ical;
use ical::parser::ParserError;
use ical::parser::ical::component::IcalCalendar;
use std::collections::HashMap;
use chrono::{DateTime, Utc, Local, TimeZone, NaiveDateTime, FixedOffset, Duration};
use std::borrow::Borrow;
use std::ops::Sub;

struct Event {
      name: String,
      location: Option<String>,
      start: DateTime<Utc>,
      end: DateTime<Utc>,
      allday: bool
}


fn main() {
      println!("e-Paper Init and Clear...");
      let mut display = Display::init();
      display.clear();
      epd::device::delay_ms(100);



      let mut cal = fetch_data();
      draw_cal(&mut display, &cal);

      let fetch_tick = chan::tick_ms(60000);
      let display_tick = chan::tick_ms(Display::update_rate());
      loop {
            chan_select! {
                  display_tick.recv() => {
                        draw_cal(&mut display, &cal);
                  },
                  fetch_tick.recv() => {
                        cal = fetch_data();
                  }
            }
      }
}

fn fetch_data() -> Vec<Event> {
      println!("Fetching cal");
      let resp = reqwest::blocking::get(&std::env::var("ICALADDR").expect("you need to set ICALADDR")).expect("Request failed").text().unwrap();
      let cal = ical::IcalParser::new(resp.as_bytes()).next().expect("Parsing failed").expect("Really failed");
      let mut output = Vec::new();

      for e in cal.events {
            let mut props = HashMap::new();

            for p in e.properties {
                  if p.value != None {
                        props.insert(p.name, p.value.unwrap());
                  }
            }

            if  props.contains_key("SUMMARY") && props.contains_key("DTEND") && props.contains_key("DTSTART") {
                  let (start, allday) = unpack_time_stamp(props.get("DTSTART"));
                  output.push(Event {
                        name: props.get("SUMMARY").unwrap().clone(),
                        location: match props.get("LOCATION") {
                              Some(s) => Some(s.clone()),
                              None => None
                        },
                        start,
                        end: unpack_time_stamp(props.get("DTEND")).0,
                        allday
                  });
            }

      }

      let now = Utc::now();
      let today = now.sub(Duration::seconds(now.timestamp() % 86400));

      let mut output = output.into_iter().filter(|e| {
            e.start >= today
      }).collect::<Vec<Event>>();

      output.sort_by(|a, b| {
            a.start.cmp(&b.start)
      });

      output
}

fn unpack_time_stamp(input: Option<&String>) -> (DateTime<Utc>, bool) {
      const FORMAT: &str = "%Y%m%dT%H%M%SZ%z";
      let input_string = input.unwrap();

      match DateTime::parse_from_str(&format!("{}{}", input_string, "+0000")[..], FORMAT) {
            Ok(d) => (d.with_timezone(&Utc), false),
            Err(_) => match DateTime::parse_from_str(&format!("{}{}", input_string, "Z+0000")[..], FORMAT) {
                  Ok(d1) => (d1.with_timezone(&Utc), false),
                  Err(_) => (DateTime::parse_from_str(&format!("{}{}", input_string, "T000000Z+0000")[..], FORMAT).unwrap().with_timezone(&Utc), true)
            }
      }
}

fn draw_cal(display: &mut Display, cal: &Vec<Event>) {

      //println!("Printing Cal {}", cal.len());
      //
      // for e in cal {
      //       println!("{:?}, {:?}", e.name, e.start);
      // }


      //println!("{:?}, {:?}", cal.first().unwrap().name, cal.first().unwrap().start);




      let mut image = epd::paint::new_image(epd::display::d7in5_v2::WIDTH, epd::display::d7in5_v2::HEIGHT, epd::paint::Color::White);

      image.clear(epd::paint::Color::White);

      let mut y: u16 = 20;
      for e in cal {
            let time = match e.allday {
                  true => format!("{} - {}", e.start.format("%d-%m-%y"), e.end.format("%d-%m-%y")),
                  false => format!("{} - {}", e.start.format("%d-%m-%y %H:%M"), e.end.format("%H:%M"))
            };

            let (_, next_y) = image.draw_string(20, y+10, &format!("{}:  {}", e.name, time)[..], epd::paint::font20(), epd::paint::Color::Black, epd::paint::Color::White);
            y = next_y;

            if e.location != None {
                  let (_, next_y) = image.draw_string(20, y, &e.location.as_ref().unwrap()[..], epd::paint::font12(), epd::paint::Color::Black, epd::paint::Color::White);
                  y = next_y;
            }
      }

      // // 2.Drawing on the image
      // image.draw_point(10, 80, epd::paint::Color::Black, epd::paint::Dot_Pixel::DOT_PIXEL_1X1, epd::paint::Dot_Style::DOT_FILL_AROUND);
      // image.draw_point(10, 90, epd::paint::Color::Black, epd::paint::Dot_Pixel::DOT_PIXEL_2X2, epd::paint::Dot_Style::DOT_FILL_AROUND);
      // image.draw_point(10, 100, epd::paint::Color::Black, epd::paint::Dot_Pixel::DOT_PIXEL_3X3, epd::paint::Dot_Style::DOT_FILL_AROUND);
      //
      // image.draw_line(20, 70, 70, 120, epd::paint::Color::Black, epd::paint::Dot_Pixel::DOT_PIXEL_1X1, epd::paint::Line_Style::LINE_STYLE_SOLID);
      // image.draw_line(70, 70, 20, 120, epd::paint::Color::Black, epd::paint::Dot_Pixel::DOT_PIXEL_1X1, epd::paint::Line_Style::LINE_STYLE_SOLID);
      //
      // image.draw_rectangle(20, 70, 70, 120, epd::paint::Color::Black, epd::paint::Dot_Pixel::DOT_PIXEL_1X1, epd::paint::Draw_Fill::DRAW_FILL_EMPTY);
      // image.draw_rectangle(80, 70, 130, 120, epd::paint::Color::Black, epd::paint::Dot_Pixel::DOT_PIXEL_1X1, epd::paint::Draw_Fill::DRAW_FILL_FULL);
      // image.draw_circle(45, 95, 20, epd::paint::Color::Black, epd::paint::Dot_Pixel::DOT_PIXEL_1X1, epd::paint::Draw_Fill::DRAW_FILL_EMPTY);
      // image.draw_circle(105, 95, 20, epd::paint::Color::White, epd::paint::Dot_Pixel::DOT_PIXEL_1X1, epd::paint::Draw_Fill::DRAW_FILL_FULL);
      // image.draw_line(85, 95, 125, 95, epd::paint::Color::Black, epd::paint::Dot_Pixel::DOT_PIXEL_1X1, epd::paint::Line_Style::LINE_STYLE_DOTTED);
      // image.draw_line(105, 75, 105, 115, epd::paint::Color::Black, epd::paint::Dot_Pixel::DOT_PIXEL_1X1, epd::paint::Line_Style::LINE_STYLE_DOTTED);
      // image.draw_string(10, 0, "waveshare", epd::paint::font16(), epd::paint::Color::Black, epd::paint::Color::White);
      // image.draw_string(10, 20, "hello world", epd::paint::font12(), epd::paint::Color::White, epd::paint::Color::Black);
      // image.draw_num(10, 33, 123456789, epd::paint::font12(), epd::paint::Color::Black, epd::paint::Color::White);
      // image.draw_num(10, 50, 987654321, epd::paint::font16(), epd::paint::Color::White, epd::paint::Color::Black);

      display.display(image);
}
