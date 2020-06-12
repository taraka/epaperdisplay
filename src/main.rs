#[macro_use]
extern crate chan;
mod epd;
use epd::display::d7in5_v2::Display;
use epd::paint::Image;
use ical;
use ical::parser::ParserError;
use ical::parser::ical::component::IcalCalendar;
use std::collections::HashMap;
use chrono::{DateTime, Utc, Local, TimeZone, NaiveDateTime, FixedOffset, Duration, Datelike};
use std::borrow::Borrow;
use std::ops::{Sub, Add};

#[derive(PartialEq)]
enum Repeat {
      NONE,
      YEARLY
}

struct Event {
      name: String,
      location: Option<String>,
      start: DateTime<Utc>,
      end: DateTime<Utc>,
      all_day: bool,
      repeat: Repeat
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
                  let repeat = get_repeat(props.get("RRULE"));

                  let (start, all_day) = unpack_time_stamp(props.get("DTSTART"), &repeat);
                  let (end, _) = unpack_time_stamp(props.get("DTEND"), &repeat);



                  output.push(Event {
                        name: props.get("SUMMARY").unwrap().clone(),
                        location: match props.get("LOCATION") {
                              Some(s) => Some(s.clone()),
                              None => None
                        },
                        start,
                        end,
                        all_day,
                        repeat
                  });
            }

      }

      let now = Utc::now();
      let today = now.sub(Duration::seconds(now.timestamp() % 86400));

      let mut output = output.into_iter().filter(|e| {
            e.start >= today || e.repeat != Repeat::NONE
      }).map(|e| {
            match e.repeat {
                  Repeat::NONE => e,
                  Repeat::YEARLY => {
                        Event {
                              name: e.name,
                              location: e.location,
                              start: find_next_yearly_instance(&e.start),
                              end: find_next_yearly_instance(&e.end),
                              all_day: e.all_day,
                              repeat: e.repeat
                        }
                  }
            }
      }).collect::<Vec<Event>>();

      output.sort_by(|a, b| {
            a.start.cmp(&b.start)
      });

      output
}

fn find_next_yearly_instance(dt: &DateTime<Utc>) -> DateTime<Utc> {
      let mut mydt = dt.clone();
      while mydt < Utc::now() {
            mydt = mydt.with_year(mydt.year() + 1).unwrap()
      }
      return mydt;
}

fn get_repeat(rrule: Option<&String>) -> Repeat {
      match rrule {
            Some(rule) => {
                  if rule == "FREQ=YEARLY" {
                        Repeat::YEARLY
                  } else {
                        Repeat::NONE
                  }
            },
            None => Repeat::NONE
      }
}

fn unpack_time_stamp(input: Option<&String>, repeat: &Repeat) -> (DateTime<Utc>, bool) {
      const FORMAT: &str = "%Y%m%dT%H%M%SZ%z";
      let input_string = input.unwrap();

      let values = match DateTime::parse_from_str(&format!("{}{}", input_string, "+0000")[..], FORMAT) {
            Ok(d) => (d.with_timezone(&Utc), false),
            Err(_) => match DateTime::parse_from_str(&format!("{}{}", input_string, "Z+0000")[..], FORMAT) {
                  Ok(d1) => (d1.with_timezone(&Utc), false),
                  Err(_) => (DateTime::parse_from_str(&format!("{}{}", input_string, "T000000Z+0000")[..], FORMAT).unwrap().with_timezone(&Utc), true)
            }
      };

      (values.0, values.1)
}

fn draw_cal(display: &mut Display, cal: &Vec<Event>) {
      let mut image = epd::paint::new_image(epd::display::d7in5_v2::WIDTH, epd::display::d7in5_v2::HEIGHT, epd::paint::Color::White);

      image.clear(epd::paint::Color::White);

      image.draw_line(105, 10, 105, 470, epd::paint::Color::Black, epd::paint::Dot_Pixel::DOT_PIXEL_1X1, epd::paint::Line_Style::LINE_STYLE_DOTTED);


      let mut y: u16 = 10;
      for e in cal {
            let end = match e.all_day {
                  true => e.end.sub(Duration::seconds(1)),
                  false => e.end
            };

            let start_date = format!("{}", e.start.format("%d/%m/%y"));
            let end_date = format!("{}", e.end.format("%d/%m/%y"));


            let time = format!("{} - {}", e.start.format("%H:%M"), end.format("%H:%M"));

            let (_, mut date_y) = image.draw_string(10, y, &format!("{}", start_date)[..], epd::paint::font16(), epd::paint::Color::Black, epd::paint::Color::White);
            if end.date() != e.start.date() {
                  let (_, end_date_y) = image.draw_string(10, date_y, &format!("{}", end_date)[..], epd::paint::font16(), epd::paint::Color::Black, epd::paint::Color::White);
                  date_y = end_date_y;
            }

            if !e.all_day {
                  let (_, time_y) = image.draw_string(10, date_y + 2, &format!("{}", time)[..], epd::paint::font12(), epd::paint::Color::Black, epd::paint::Color::White);
                  date_y = time_y
            }
            let (_, next_y) = image.draw_string(115, y, &format!("{}", e.name)[..], epd::paint::font20(), epd::paint::Color::Black, epd::paint::Color::White);
            y = next_y;

            if e.location != None {
                  let (_, next_y) = image.draw_string(115, y+2, &e.location.as_ref().unwrap().replace("\\n", ", ").replace("\\", " ")[..], epd::paint::font12(), epd::paint::Color::Black, epd::paint::Color::White);
                  y = next_y;
            }

            y = if y > date_y { y } else { date_y };

            image.draw_line(10, y+8, 790, y+8, epd::paint::Color::Black, epd::paint::Dot_Pixel::DOT_PIXEL_1X1, epd::paint::Line_Style::LINE_STYLE_DOTTED);
            y+=16
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
