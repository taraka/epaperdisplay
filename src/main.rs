mod epd;

use epd::display::d7in5_v2::Display;

use ical;
use chrono::{DateTime, Utc, Duration, Datelike, NaiveDate};
use chan::chan_select;
use serde::Deserialize;

use std::ops::Sub;
use std::collections::HashMap;


#[derive(PartialEq, Clone)]
enum Repeat {
      NONE,
      YEARLY,
      WEEKLY,
      MONTHLY,
}

struct Event {
      name: String,
      location: Option<String>,
      start: DateTime<Utc>,
      end: DateTime<Utc>,
      all_day: bool,
      repeat: Repeat,
      is_recurring: bool,
}

struct WeatherData {
      temperature: i32,
      condition: &'static str,
      weathercode: u32,
      forecast: Vec<DailyForecast>,
}

struct DailyForecast {
      date: NaiveDate,
      temp_max: i32,
      weathercode: u32,
}

#[derive(Deserialize)]
struct GeoResponse {
      results: Option<Vec<GeoLocation>>,
}

#[derive(Deserialize)]
struct GeoLocation {
      latitude: f64,
      longitude: f64,
}

#[derive(Deserialize)]
struct WeatherResponse {
      current_weather: CurrentWeather,
      daily: Option<DailyData>,
}

#[derive(Deserialize)]
struct DailyData {
      time: Vec<String>,
      weathercode: Vec<u32>,
      temperature_2m_max: Vec<f64>,
}

#[derive(Deserialize)]
struct CurrentWeather {
      temperature: f64,
      weathercode: u32,
}

fn wmo_condition(code: u32) -> &'static str {
      match code {
            0       => "Clear",
            1..=3   => "Cloudy",
            45 | 48 => "Fog",
            51..=57 => "Drizzle",
            61..=67 => "Rain",
            71..=77 => "Snow",
            80..=82 => "Showers",
            85 | 86 => "Snow showers",
            95..=99 => "Thunder",
            _       => "",
      }
}

fn geocode_town() -> Option<(f64, f64)> {
      let town = std::env::var("TOWN").ok()?;
      let geo_url = format!(
            "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1", town
      );
      let loc = reqwest::blocking::get(&geo_url).ok()?
            .json::<GeoResponse>().ok()?
            .results?
            .into_iter().next()?;
      Some((loc.latitude, loc.longitude))
}

fn fetch_weather(lat: f64, lon: f64) -> Option<WeatherData> {
      let weather_url = format!(
            "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current_weather=true&daily=weathercode,temperature_2m_max&timezone=UTC&forecast_days=14",
            lat, lon
      );
      let resp = reqwest::blocking::get(&weather_url).ok()?
            .json::<WeatherResponse>().ok()?;

      let cw = resp.current_weather;

      let forecast = resp.daily.map(|d| {
            d.time.iter().zip(d.weathercode.iter()).zip(d.temperature_2m_max.iter())
                  .filter_map(|((date_str, &code), &temp)| {
                        NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok().map(|date| DailyForecast {
                              date,
                              temp_max: temp.round() as i32,
                              weathercode: code,
                        })
                  }).collect()
      }).unwrap_or_default();

      Some(WeatherData {
            temperature: cw.temperature.round() as i32,
            condition: wmo_condition(cw.weathercode),
            weathercode: cw.weathercode,
            forecast,
      })
}


fn main() {
      println!("e-Paper Init and Clear...");
      let mut display = Display::init();
      display.clear();

      let location = geocode_town();
      if location.is_none() && std::env::var("TOWN").is_ok() {
            println!("Warning: could not geocode TOWN, weather disabled");
      }

      let mut cal = fetch_data();
      let mut weather = location.and_then(|(lat, lon)| fetch_weather(lat, lon));
      if weather.is_none() && location.is_some() {
            println!("Warning: weather fetch failed");
      }
      draw_cal(&mut display, &cal, weather.as_ref());

      let fetch_tick = chan::tick_ms(5 * 60 * 1000);
      let display_tick = chan::tick_ms(Display::update_rate());
      loop {
            chan_select! {
                  display_tick.recv() => {
                        draw_cal(&mut display, &cal, weather.as_ref());
                  },
                  fetch_tick.recv() => {
                        cal = fetch_data();
                        weather = location.and_then(|(lat, lon)| fetch_weather(lat, lon));
                        if weather.is_none() && location.is_some() {
                              println!("Warning: weather fetch failed");
                        }
                        draw_cal(&mut display, &cal, weather.as_ref());
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

                  let (start, all_day) = match unpack_time_stamp(props.get("DTSTART")) {
                        Some(v) => v,
                        None => continue,
                  };
                  let (end, _) = match unpack_time_stamp(props.get("DTEND")) {
                        Some(v) => v,
                        None => continue,
                  };



                  output.push(Event {
                        name: props.get("SUMMARY").unwrap().clone(),
                        location: match props.get("LOCATION") {
                              Some(s) => Some(s.clone()),
                              None => None
                        },
                        start,
                        end,
                        all_day,
                        is_recurring: false,
                        repeat
                  });
            }

      }

      let now = Utc::now();
      let today_start = now.sub(Duration::seconds(now.timestamp() % 86400));
      let lookahead = today_start + Duration::weeks(8);

      let mut output = output.into_iter().filter(|e| {
            e.start >= today_start || e.repeat != Repeat::NONE
      }).flat_map(|e| {
            match e.repeat {
                  Repeat::NONE => vec![e],
                  Repeat::YEARLY => vec![Event {
                        start: find_next_yearly_instance(&e.start),
                        end: find_next_yearly_instance(&e.end),
                        name: e.name,
                        location: e.location,
                        all_day: e.all_day,
                        is_recurring: false,
                        repeat: e.repeat,
                  }],
                  Repeat::WEEKLY => {
                        let duration = e.end - e.start;
                        let mut dt = e.start;
                        while dt < today_start {
                              dt = dt + Duration::weeks(1);
                        }
                        let mut instances = Vec::new();
                        while dt <= lookahead {
                              instances.push(Event {
                                    name: e.name.clone(),
                                    location: e.location.clone(),
                                    start: dt,
                                    end: dt + duration,
                                    all_day: e.all_day,
                                    is_recurring: true,
                                    repeat: Repeat::NONE,
                              });
                              dt = dt + Duration::weeks(1);
                        }
                        instances
                  },
                  Repeat::MONTHLY => {
                        let duration = e.end - e.start;
                        let mut dt = e.start;
                        while dt < today_start {
                              dt = add_one_month(dt);
                        }
                        let mut instances = Vec::new();
                        while dt <= lookahead {
                              instances.push(Event {
                                    name: e.name.clone(),
                                    location: e.location.clone(),
                                    start: dt,
                                    end: dt + duration,
                                    all_day: e.all_day,
                                    is_recurring: true,
                                    repeat: Repeat::NONE,
                              });
                              dt = add_one_month(dt);
                        }
                        instances
                  }
            }
      }).collect::<Vec<Event>>();

      output.sort_by(|a, b| {
            a.start.cmp(&b.start).then(a.name.cmp(&b.name))
      });

      output
}

fn add_one_month(dt: DateTime<Utc>) -> DateTime<Utc> {
      let (year, month) = if dt.month() == 12 {
            (dt.year() + 1, 1)
      } else {
            (dt.year(), dt.month() + 1)
      };
      dt.with_year(year).and_then(|d| d.with_month(month))
            .unwrap_or_else(|| dt + Duration::days(28))
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

fn repeat_expired(rule: &str) -> bool {
      if let Some(idx) = rule.find("UNTIL=") {
            let until_val = rule[idx + 6..].split(';').next().unwrap_or("");
            if until_val.len() >= 8 {
                  if let Ok(until_date) = NaiveDate::parse_from_str(&until_val[..8], "%Y%m%d") {
                        return until_date < Utc::now().date_naive();
                  }
            }
      }
      false
}

fn get_repeat(rrule: Option<&String>) -> Repeat {
      match rrule {
            Some(rule) => {
                  if rule.starts_with("FREQ=YEARLY") {
                        if repeat_expired(rule) { Repeat::NONE } else { Repeat::YEARLY }
                  } else if rule.starts_with("FREQ=WEEKLY") {
                        if repeat_expired(rule) { Repeat::NONE } else { Repeat::WEEKLY }
                  } else if rule.starts_with("FREQ=MONTHLY") {
                        if repeat_expired(rule) { Repeat::NONE } else { Repeat::MONTHLY }
                  } else {
                        Repeat::NONE
                  }
            },
            None => Repeat::NONE
      }
}

fn unpack_time_stamp(input: Option<&String>) -> Option<(DateTime<Utc>, bool)> {
      const FORMAT: &str = "%Y%m%dT%H%M%SZ%z";
      let input_string = input?;
      let result = match DateTime::parse_from_str(&format!("{}{}", input_string, "+0000")[..], FORMAT) {
            Ok(d) => (d.with_timezone(&Utc), false),
            Err(_) => match DateTime::parse_from_str(&format!("{}{}", input_string, "Z+0000")[..], FORMAT) {
                  Ok(d1) => (d1.with_timezone(&Utc), false),
                  Err(_) => match DateTime::parse_from_str(&format!("{}{}", input_string, "T000000Z+0000")[..], FORMAT) {
                        Ok(d2) => (d2.with_timezone(&Utc), true),
                        Err(e) => {
                              println!("Warning: could not parse timestamp {:?}: {}", input_string, e);
                              return None;
                        }
                  }
            }
      };
      Some(result)
}

// Large cloud (for 28px icon).
fn draw_cloud_shape(image: &mut epd::paint::Image, cx: u16, cy: u16, color: epd::paint::Color) {
      let px1 = epd::paint::DotPixel::DotPixel1x1;
      let fill = epd::paint::DrawFill::DrawFillFull;
      image.draw_circle(cx.saturating_sub(7), cy, 4, color, px1, fill);
      image.draw_circle(cx, cy.saturating_sub(4), 6, color, px1, fill);
      image.draw_circle(cx + 7, cy.saturating_sub(1), 4, color, px1, fill);
      image.draw_rectangle(cx.saturating_sub(11), cy, cx + 11, cy + 5, color, px1, fill);
}

// Small cloud (for 20px icon): two circles give a cleaner shape at this scale.
fn draw_cloud_shape_small(image: &mut epd::paint::Image, cx: u16, cy: u16, color: epd::paint::Color) {
      let px1 = epd::paint::DotPixel::DotPixel1x1;
      let fill = epd::paint::DrawFill::DrawFillFull;
      image.draw_circle(cx.saturating_sub(4), cy.saturating_sub(1), 3, color, px1, fill); // left bump
      image.draw_circle(cx + 1, cy.saturating_sub(3), 4, color, px1, fill);               // main dome
      image.draw_rectangle(cx.saturating_sub(7), cy + 1, cx + 5, cy + 3, color, px1, fill); // flat body
}

fn draw_weather_icon_small(image: &mut epd::paint::Image, x: u16, y: u16, code: u32, color: epd::paint::Color) {
      let px1 = epd::paint::DotPixel::DotPixel1x1;
      let px2 = epd::paint::DotPixel::DotPixel2x2;
      let solid = epd::paint::LineStyle::LineStyleSolid;
      let fill = epd::paint::DrawFill::DrawFillFull;

      let cx = x + 10;
      let cy = y + 10;

      match code {
            0 => {
                  image.draw_circle(cx, cy, 3, color, px1, fill);
                  for &(x1, y1, x2, y2) in &[
                        (0i16, -5, 0i16, -7), (0, 5, 0, 7), (-5, 0, -7, 0), (5, 0, 7, 0),
                        (-4, -4, -5, -5), (4, -4, 5, -5), (-4, 4, -5, 5), (4, 4, 5, 5),
                  ] {
                        image.draw_line(
                              (cx as i16 + x1) as u16, (cy as i16 + y1) as u16,
                              (cx as i16 + x2) as u16, (cy as i16 + y2) as u16,
                              color, px1, solid,
                        );
                  }
            }
            1..=3 => {
                  draw_cloud_shape_small(image, cx, cy, color);
            }
            45 | 48 => {
                  for i in 0..3u16 {
                        image.draw_line(cx.saturating_sub(7), cy.saturating_sub(4) + i * 4,
                              cx + 7, cy.saturating_sub(4) + i * 4, color, px1, solid);
                  }
            }
            51..=57 | 61..=67 | 80..=82 => {
                  draw_cloud_shape_small(image, cx, cy.saturating_sub(3), color);
                  for i in 0..3u16 {
                        let rx = cx.saturating_sub(4) + i * 4;
                        image.draw_line(rx, cy + 3, rx.saturating_sub(1), cy + 6, color, px1, solid);
                  }
            }
            71..=77 | 85 | 86 => {
                  draw_cloud_shape_small(image, cx, cy.saturating_sub(3), color);
                  for i in 0..3u16 {
                        image.draw_circle(cx.saturating_sub(4) + i * 4, cy + 6, 1, color, px1, fill);
                  }
            }
            95..=99 => {
                  draw_cloud_shape_small(image, cx, cy.saturating_sub(4), color);
                  image.draw_line(cx + 1, cy + 2, cx.saturating_sub(1), cy + 5, color, px2, solid);
                  image.draw_line(cx.saturating_sub(1), cy + 5, cx + 2, cy + 8, color, px2, solid);
            }
            _ => {}
      }
}

fn draw_weather_icon(image: &mut epd::paint::Image, x: u16, y: u16, code: u32) {
      let color = epd::paint::Color::White;
      let px1 = epd::paint::DotPixel::DotPixel1x1;
      let px2 = epd::paint::DotPixel::DotPixel2x2;
      let solid = epd::paint::LineStyle::LineStyleSolid;
      let fill = epd::paint::DrawFill::DrawFillFull;

      let cx = x + 14;
      let cy = y + 14;

      match code {
            0 => {
                  // Sun: disc + 8 rays
                  image.draw_circle(cx, cy, 5, color, px1, fill);
                  for &(x1, y1, x2, y2) in &[
                        (0i16, -8, 0i16, -11), (0, 8, 0, 11), (-8, 0, -11, 0), (8, 0, 11, 0),
                        (-6, -6, -8, -8), (6, -6, 8, -8), (-6, 6, -8, 8), (6, 6, 8, 8),
                  ] {
                        image.draw_line(
                              (cx as i16 + x1) as u16, (cy as i16 + y1) as u16,
                              (cx as i16 + x2) as u16, (cy as i16 + y2) as u16,
                              color, px2, solid,
                        );
                  }
            }
            1..=3 => {
                  draw_cloud_shape(image, cx, cy, color);
            }
            45 | 48 => {
                  // Fog: three horizontal bars
                  for i in 0..3u16 {
                        image.draw_line(cx.saturating_sub(10), cy.saturating_sub(5) + i * 5,
                              cx + 10, cy.saturating_sub(5) + i * 5, color, px2, solid);
                  }
            }
            51..=57 | 61..=67 | 80..=82 => {
                  // Rain: cloud + diagonal drops
                  draw_cloud_shape(image, cx, cy.saturating_sub(5), color);
                  for i in 0..4u16 {
                        let rx = cx.saturating_sub(6) + i * 4;
                        image.draw_line(rx, cy + 3, rx.saturating_sub(2), cy + 8, color, px1, solid);
                  }
            }
            71..=77 | 85 | 86 => {
                  // Snow: cloud + dots
                  draw_cloud_shape(image, cx, cy.saturating_sub(5), color);
                  for i in 0..4u16 {
                        image.draw_circle(cx.saturating_sub(6) + i * 4, cy + 7, 1, color, px1, fill);
                  }
            }
            95..=99 => {
                  // Thunder: cloud + lightning bolt
                  draw_cloud_shape(image, cx, cy.saturating_sub(6), color);
                  image.draw_line(cx + 2, cy + 2, cx.saturating_sub(2), cy + 6, color, px2, solid);
                  image.draw_line(cx.saturating_sub(2), cy + 6, cx + 3, cy + 11, color, px2, solid);
            }
            _ => {}
      }
}

fn draw_cal(display: &mut Display, cal: &Vec<Event>, weather: Option<&WeatherData>) {
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
      if let Some(w) = weather {
            let weather_str = format!("{}C  {}", w.temperature, w.condition);
            let text_w = weather_str.len() as u16 * 14; // FONT20 is 14px wide
            let icon_x = (790u16).saturating_sub(text_w + 6 + 28);
            let text_x = icon_x + 28 + 6;
            draw_weather_icon(&mut image, icon_x, 4, w.weathercode);
            image.draw_string(text_x, 8, &weather_str,
                  &epd::font::FONT20, epd::paint::Color::White, epd::paint::Color::Black);
      }

      // Vertical divider
      image.draw_line(DIVIDER_X, HEADER_H, DIVIDER_X, epd::display::d7in5_v2::HEIGHT,
            epd::paint::Color::Black, epd::paint::DotPixel::DotPixel1x1, epd::paint::LineStyle::LineStyleSolid);

      let forecast_map: HashMap<NaiveDate, &DailyForecast> = weather
            .map(|w| w.forecast.iter().map(|f| (f.date, f)).collect())
            .unwrap_or_default();

      let mut y: u16 = HEADER_H + 8;
      for e in cal {
            if y + 24 >= epd::display::d7in5_v2::HEIGHT { break; }
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

            let name_font = if e.is_recurring { &epd::font::FONT16 } else { &epd::font::FONT24 };
            let name_font_h: u16 = if e.is_recurring { 16 } else { 24 };

            // Pre-calculate row height so we can draw the inverted background first
            let date_h: u16 = if e.is_recurring {
                  12 // date + time combined on one FONT12 line
            } else {
                  let mut h = 16u16;
                  if end.date_naive() != e.start.date_naive() { h += 16; }
                  if !e.all_day { h += 14; }
                  h
            };
            let mut name_h: u16 = name_font_h;
            if !e.is_recurring { if e.location.is_some() { name_h += 14; } }
            let row_h = date_h.max(name_h);

            if is_today {
                  image.draw_rectangle(0, y.saturating_sub(4), epd::display::d7in5_v2::WIDTH, y + row_h + 4,
                        epd::paint::Color::Black, epd::paint::DotPixel::DotPixel1x1, epd::paint::DrawFill::DrawFillFull);
                  image.draw_line(DIVIDER_X, y.saturating_sub(4), DIVIDER_X, y + row_h + 4,
                        epd::paint::Color::White, epd::paint::DotPixel::DotPixel1x1, epd::paint::LineStyle::LineStyleSolid);
            }

            // Left column: date and time
            let (_, date_y) = if e.is_recurring {
                  let date_time_str = if e.all_day {
                        e.start.format("%a %d %b").to_string()
                  } else {
                        format!("{} {}", e.start.format("%a %d %b"), e.start.format("%H:%M"))
                  };
                  image.draw_string(10, y, &date_time_str, &epd::font::FONT12, fg, bg)
            } else {
                  let (_, mut dy) = image.draw_string(10, y, &e.start.format("%a %d %b").to_string(),
                        &epd::font::FONT16, fg, bg);
                  if end.date_naive() != e.start.date_naive() {
                        let (_, edy) = image.draw_string(10, dy, &e.end.format("%a %d %b").to_string(),
                              &epd::font::FONT16, fg, bg);
                        dy = edy;
                  }
                  if !e.all_day {
                        let time_str = format!("{} - {}", e.start.format("%H:%M"), end.format("%H:%M"));
                        let (_, ty) = image.draw_string(10, dy + 2, &time_str, &epd::font::FONT12, fg, bg);
                        dy = ty;
                  }
                  (0, dy)
            };

            // Right column: name (and location for non-recurring)
            // Truncate name so it doesn't overflow into the forecast icon area (~50px on the right)
            let max_name_chars = ((790u16.saturating_sub(DIVIDER_X + 10 + 50)) / name_font.width) as usize;
            let display_name: String = e.name.chars().take(max_name_chars).collect();
            let (_, mut name_y) = image.draw_string(DIVIDER_X + 10, y, &display_name, name_font, fg, bg);
            if !e.is_recurring {
                  if let Some(loc) = &e.location {
                        let (_, ly) = image.draw_string(DIVIDER_X + 10, name_y + 2,
                              &loc.replace("\\n", ", ").replace("\\", " "), &epd::font::FONT12, fg, bg);
                        name_y = ly;
                  }
            }

            // Forecast icon + max temp, right-aligned.
            // For multi-day events ongoing today, look up today's forecast rather than the (past) start date.
            let forecast_date = if is_today && e.start.date_naive() < today {
                  today
            } else {
                  e.start.date_naive()
            };
            if let Some(f) = forecast_map.get(&forecast_date) {
                  let temp_str = format!("{}C", f.temp_max);
                  let temp_w = temp_str.len() as u16 * 7; // FONT12 is 7px wide
                  let icon_x = (790u16).saturating_sub(temp_w + 4 + 20);
                  let temp_x = icon_x + 20 + 4;
                  draw_weather_icon_small(&mut image, icon_x, y + 2, f.weathercode, fg);
                  image.draw_string(temp_x, y + 6, &temp_str, &epd::font::FONT12, fg, bg);
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
