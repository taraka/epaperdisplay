mod epd;

use epd::display::d7in5_v2::Display;

use ical;
use chrono::{DateTime, Utc, Duration, Datelike};
use chan::chan_select;

use std::ops::Sub;
use std::collections::HashMap;


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

      let mut cal = fetch_data();
      draw_cal(&mut display, &cal);

      let fetch_tick = chan::tick_ms(5 * 60 * 1000);
      let display_tick = chan::tick_ms(Display::update_rate());
      loop {
            chan_select! {
                  display_tick.recv() => {
                        draw_cal(&mut display, &cal);
                  },
                  fetch_tick.recv() => {
                        cal = fetch_data();
                        draw_cal(&mut display, &cal);
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

                  let (start, all_day) = unpack_time_stamp(props.get("DTSTART"));
                  let (end, _) = unpack_time_stamp(props.get("DTEND"));



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
      let today: DateTime<Utc> = now.sub(Duration::seconds(now.timestamp() % 86400)).sub(Duration::seconds(1));

      //println!("{:?}", today);

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
            a.start.cmp(&b.start).then(a.name.cmp(&b.name))
      });

      output
}

fn find_next_yearly_instance(dt: &DateTime<Utc>) -> DateTime<Utc> {
      let now = Utc::now();
      let today_start = now.sub(Duration::seconds(now.timestamp() % 86400));
      let mut mydt = dt.clone();
      while mydt < today_start {
            mydt = mydt.with_year(mydt.year() + 1).unwrap()
      }
      return mydt;
}

fn get_repeat(rrule: Option<&String>) -> Repeat {
      match rrule {
            Some(rule) => {
                  if &rule[0..11] == "FREQ=YEARLY" {
                        Repeat::YEARLY
                  } else {
                        //eprintln!("Invalid repeat: {:?}", &rrule);
                        Repeat::NONE
                  }
            },
            None => Repeat::NONE
      }
}

fn unpack_time_stamp(input: Option<&String>) -> (DateTime<Utc>, bool) {
      const FORMAT: &str = "%Y%m%dT%H%M%SZ%z";
      let input_string = input.unwrap();
      //println!("{}", input_string);
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
      const HEADER_H: u16 = 36;
      const DIVIDER_X: u16 = 130;

      let mut image = epd::paint::new_image(epd::display::d7in5_v2::WIDTH, epd::display::d7in5_v2::HEIGHT, epd::paint::Color::White);
      image.clear(epd::paint::Color::White);

      let now = Utc::now();
      let today = now.date_naive();

      // Header bar
      image.draw_rectangle(0, 0, epd::display::d7in5_v2::WIDTH, HEADER_H,
            epd::paint::Color::Black, epd::paint::DotPixel::DotPixel1x1, epd::paint::DrawFill::DrawFillFull);
      image.draw_string(10, 8, &now.format("%A %d %B %Y").to_string(),
            &epd::font::FONT20, epd::paint::Color::White, epd::paint::Color::Black);

      // Vertical divider
      image.draw_line(DIVIDER_X, HEADER_H, DIVIDER_X, epd::display::d7in5_v2::HEIGHT,
            epd::paint::Color::Black, epd::paint::DotPixel::DotPixel1x1, epd::paint::LineStyle::LineStyleSolid);

      let mut y: u16 = HEADER_H + 8;
      for e in cal {
            let end = match e.all_day {
                  true => e.end.sub(Duration::seconds(1)),
                  false => e.end
            };

            let is_today = e.start.date_naive() == today ||
                  (e.start.date_naive() <= today && end.date_naive() >= today);

            let (fg, bg) = if is_today {
                  (epd::paint::Color::White, epd::paint::Color::Black)
            } else {
                  (epd::paint::Color::Black, epd::paint::Color::White)
            };

            // Pre-calculate row height so we can draw the inverted background first
            let mut date_h: u16 = 16;
            if end.date_naive() != e.start.date_naive() { date_h += 16; }
            if !e.all_day { date_h += 14; }
            let mut name_h: u16 = 24;
            if e.location.is_some() { name_h += 14; }
            let row_h = date_h.max(name_h);

            if is_today {
                  image.draw_rectangle(0, y.saturating_sub(4), epd::display::d7in5_v2::WIDTH, y + row_h + 4,
                        epd::paint::Color::Black, epd::paint::DotPixel::DotPixel1x1, epd::paint::DrawFill::DrawFillFull);
                  image.draw_line(DIVIDER_X, y.saturating_sub(4), DIVIDER_X, y + row_h + 4,
                        epd::paint::Color::White, epd::paint::DotPixel::DotPixel1x1, epd::paint::LineStyle::LineStyleSolid);
            }

            // Left column: date and time
            let (_, mut date_y) = image.draw_string(10, y, &e.start.format("%a %d %b").to_string(),
                  &epd::font::FONT16, fg, bg);
            if end.date_naive() != e.start.date_naive() {
                  let (_, edy) = image.draw_string(10, date_y, &e.end.format("%a %d %b").to_string(),
                        &epd::font::FONT16, fg, bg);
                  date_y = edy;
            }
            if !e.all_day {
                  let time_str = format!("{} - {}", e.start.format("%H:%M"), end.format("%H:%M"));
                  let (_, ty) = image.draw_string(10, date_y + 2, &time_str, &epd::font::FONT12, fg, bg);
                  date_y = ty;
            }

            // Right column: name and location
            let (_, mut name_y) = image.draw_string(DIVIDER_X + 10, y, &e.name, &epd::font::FONT24, fg, bg);
            if let Some(loc) = &e.location {
                  let (_, ly) = image.draw_string(DIVIDER_X + 10, name_y + 2,
                        &loc.replace("\\n", ", ").replace("\\", " "), &epd::font::FONT12, fg, bg);
                  name_y = ly;
            }

            y = date_y.max(name_y);

            image.draw_line(10, y + 8, 790, y + 8,
                  epd::paint::Color::Black, epd::paint::DotPixel::DotPixel1x1, epd::paint::LineStyle::LineStyleSolid);
            y += 16;
      }

      if display.display(image) {
            //println!("Display updated");
      }
}
