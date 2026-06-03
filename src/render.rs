use chrono::{Duration, Local};
use std::collections::HashMap;

use crate::calendar::Event;
use crate::epd;
use crate::epd::display::{Display, HEIGHT, WIDTH};
use crate::weather::{DailyForecast, WeatherData};

const HEADER_H: u16 = 36;
const FOOTER_H: u16 = 22;
const DIVIDER_X: u16 = 130;

pub enum WeatherStatus<'a> {
    Available(&'a WeatherData),
    Unavailable,
    Disabled,
}

// ── Header ────────────────────────────────────────────────────────────────────

fn draw_header(image: &mut epd::paint::Image, weather: &WeatherStatus) {
    image.draw_rectangle(
        0, 0, WIDTH, HEADER_H,
        epd::paint::Color::Black,
        epd::paint::DotPixel::DotPixel1x1,
        epd::paint::DrawFill::DrawFillFull,
    );

    let now = Local::now();
    image.draw_string(
        10, 8, &now.format("%A %d %B %Y").to_string(),
        &epd::font::FONT20, epd::paint::Color::White, epd::paint::Color::Black,
    );

    match weather {
        WeatherStatus::Available(w) => {
            let weather_str = format!("{}C  {}", w.temperature, w.condition);
            let text_w = weather_str.len() as u16 * epd::font::FONT20.width;
            let icon_x = (790u16).saturating_sub(text_w + 6 + 28);
            let text_x = icon_x + 28 + 6;
            draw_weather_icon(image, icon_x, 4, w.weathercode, epd::paint::Color::White, IconSize::Large);
            image.draw_string(
                text_x, 8, &weather_str,
                &epd::font::FONT20, epd::paint::Color::White, epd::paint::Color::Black,
            );
        }
        WeatherStatus::Unavailable => {
            let label = "Weather unavailable";
            let x = (790u16).saturating_sub(label.len() as u16 * epd::font::FONT20.width);
            image.draw_string(x, 8, label, &epd::font::FONT20, epd::paint::Color::White, epd::paint::Color::Black);
        }
        WeatherStatus::Disabled => {}
    }
}

// ── Public drawing functions ──────────────────────────────────────────────────

pub fn draw_error(display: &mut Display, message: &str) {
    log::warn!("Displaying error on screen: {}", message);

    let mut image = epd::paint::new_image(WIDTH, HEIGHT, epd::paint::Color::White);
    image.clear(epd::paint::Color::White);

    draw_header(&mut image, &WeatherStatus::Disabled);

    image.draw_string(
        20, HEADER_H + 20, "Error",
        &epd::font::FONT24, epd::paint::Color::Black, epd::paint::Color::White,
    );
    image.draw_line(
        20, HEADER_H + 48, WIDTH - 20, HEADER_H + 48,
        epd::paint::Color::Black,
        epd::paint::DotPixel::DotPixel1x1,
        epd::paint::LineStyle::LineStyleSolid,
    );
    image.draw_string(
        20, HEADER_H + 58, message,
        &epd::font::FONT16, epd::paint::Color::Black, epd::paint::Color::White,
    );

    display.display(image);
}

pub fn draw_cal(display: &mut Display, cal: &[Event], weather: WeatherStatus) {
    let mut image = epd::paint::new_image(WIDTH, HEIGHT, epd::paint::Color::White);
    image.clear(epd::paint::Color::White);

    draw_header(&mut image, &weather);

    image.draw_line(
        DIVIDER_X, HEADER_H, DIVIDER_X, HEIGHT - FOOTER_H,
        epd::paint::Color::Black,
        epd::paint::DotPixel::DotPixel1x1,
        epd::paint::LineStyle::LineStyleSolid,
    );

    let today = Local::now().date_naive();

    let forecast_map: HashMap<chrono::NaiveDate, &DailyForecast> = match &weather {
        WeatherStatus::Available(w) => w.forecast.iter().map(|f| (f.date, f)).collect(),
        _ => HashMap::new(),
    };

    let mut y: u16 = HEADER_H + 8;
    let mut events_drawn: usize = 0;

    for e in cal {
        let start_local = e.start.with_timezone(&Local);
        let end_local = match e.all_day {
            true => (e.end - Duration::seconds(1)).with_timezone(&Local),
            false => e.end.with_timezone(&Local),
        };

        let is_today = start_local.date_naive() == today
            || (start_local.date_naive() <= today && end_local.date_naive() >= today);

        let fg = epd::paint::Color::Black;
        let bg = epd::paint::Color::White;

        let name_font = match (e.is_recurring, is_today) {
            (true,  true)  => &epd::font::FONT20,
            (true,  false) => &epd::font::FONT16,
            (false, _)     => &epd::font::FONT24,
        };
        let name_font_h: u16 = match (e.is_recurring, is_today) {
            (true,  true)  => 20,
            (true,  false) => 16,
            (false, _)     => 24,
        };
        let date_font = if e.is_recurring { &epd::font::FONT12 } else { &epd::font::FONT16 };
        let date_small_font = &epd::font::FONT12;

        let date_h: u16 = if e.is_recurring {
            12
        } else {
            let mut h = 16u16;
            if end_local.date_naive() != start_local.date_naive() { h += 16; }
            if !e.all_day { h += 14; }
            h
        };
        let mut name_h = name_font_h;
        if !e.is_recurring && e.location.is_some() { name_h += 14; }
        let row_h = date_h.max(name_h);

        // row_h + separator gap + some breathing room must fit before the footer
        if y + row_h + 18 >= HEIGHT - FOOTER_H {
            break;
        }

        // Left column: date and time
        let (_, date_y) = if e.is_recurring {
            let date_time_str = if e.all_day {
                start_local.format("%a %d %b").to_string()
            } else {
                format!("{} {}", start_local.format("%a %d %b"), start_local.format("%H:%M"))
            };
            image.draw_string(10, y, &date_time_str, date_font, fg, bg)
        } else {
            let (_, mut dy) = image.draw_string(
                10, y, &start_local.format("%a %d %b").to_string(), date_font, fg, bg,
            );
            if end_local.date_naive() != start_local.date_naive() {
                let (_, edy) = image.draw_string(
                    10, dy, &end_local.format("%a %d %b").to_string(), date_font, fg, bg,
                );
                dy = edy;
            }
            if !e.all_day {
                let time_str = format!("{} - {}", start_local.format("%H:%M"), end_local.format("%H:%M"));
                let (_, ty) = image.draw_string(10, dy + 2, &time_str, date_small_font, fg, bg);
                dy = ty;
            }
            (0u16, dy)
        };

        // Right column: name vertically centered within the row, truncated to avoid forecast icon
        let name_y_start = y + (row_h.saturating_sub(name_h)) / 2;
        let max_name_chars = ((790u16.saturating_sub(DIVIDER_X + 10 + 50)) / name_font.width) as usize;
        let display_name: String = if e.name.chars().count() > max_name_chars {
            e.name.chars().take(max_name_chars.saturating_sub(1)).collect::<String>() + "…"
        } else {
            e.name.chars().take(max_name_chars).collect()
        };
        let (_, mut name_y) = image.draw_string(DIVIDER_X + 10, name_y_start, &display_name, name_font, fg, bg);
        if !e.is_recurring {
            if let Some(loc) = &e.location {
                let (_, ly) = image.draw_string(
                    DIVIDER_X + 10, name_y,
                    &loc.replace("\\n", ", ").replace("\\", " "),
                    &epd::font::FONT12, fg, bg,
                );
                name_y = ly;
            }
        }

        // Forecast icon + max temp, right-aligned
        let forecast_date = if is_today && start_local.date_naive() < today {
            today
        } else {
            start_local.date_naive()
        };
        if let Some(f) = forecast_map.get(&forecast_date) {
            let temp_str = format!("{}C", f.temp_max);
            let temp_w = temp_str.len() as u16 * epd::font::FONT12.width;
            let icon_x = (790u16).saturating_sub(temp_w + 4 + 20);
            let temp_x = icon_x + 20 + 4;
            let icon_y = y + (row_h.saturating_sub(20)) / 2;
            draw_weather_icon(&mut image, icon_x, icon_y, f.weathercode, fg, IconSize::Small);
            image.draw_string(temp_x, y + (row_h.saturating_sub(12)) / 2, &temp_str, &epd::font::FONT12, fg, bg);
        }

        events_drawn += 1;
        y = date_y.max(name_y);

        let (line_px, line_gap) = if is_today {
            (epd::paint::DotPixel::DotPixel2x2, 18)
        } else {
            (epd::paint::DotPixel::DotPixel1x1, 16)
        };
        image.draw_line(
            10, y + 8, 790, y + 8,
            epd::paint::Color::Black,
            line_px,
            epd::paint::LineStyle::LineStyleSolid,
        );
        y += line_gap;
    }

    draw_footer(&mut image, &weather);

    let updated = display.display(image);
    if updated {
        log::info!("Display refreshed ({} event(s) shown)", events_drawn);
    } else {
        log::debug!("Display skipped — image unchanged");
    }
}

fn draw_footer(image: &mut epd::paint::Image, weather: &WeatherStatus) {
    let y = HEIGHT - FOOTER_H;
    image.draw_rectangle(
        0, y, WIDTH, HEIGHT,
        epd::paint::Color::Black,
        epd::paint::DotPixel::DotPixel1x1,
        epd::paint::DrawFill::DrawFillFull,
    );

    let now = Local::now();
    let updated = format!("Updated {}", now.format("%H:%M"));
    image.draw_string(10, y + 4, &updated, &epd::font::FONT12, epd::paint::Color::White, epd::paint::Color::Black);

    if let WeatherStatus::Available(w) = weather {
        if let (Some(rise), Some(set)) = (&w.sunrise, &w.sunset) {
            let sun_str = format!("Sunrise {}  Sunset {}", rise, set);
            let text_w = sun_str.len() as u16 * epd::font::FONT12.width;
            image.draw_string(WIDTH - text_w - 10, y + 4, &sun_str, &epd::font::FONT12, epd::paint::Color::White, epd::paint::Color::Black);
        }
    }
}

// ── Weather icons ─────────────────────────────────────────────────────────────

enum IconSize {
    Small,
    Large,
}

fn draw_cloud_shape(image: &mut epd::paint::Image, cx: u16, cy: u16, color: epd::paint::Color) {
    let px1 = epd::paint::DotPixel::DotPixel1x1;
    let fill = epd::paint::DrawFill::DrawFillFull;
    image.draw_circle(cx.saturating_sub(7), cy, 4, color, px1, fill);
    image.draw_circle(cx, cy.saturating_sub(4), 6, color, px1, fill);
    image.draw_circle(cx + 7, cy.saturating_sub(1), 4, color, px1, fill);
    image.draw_rectangle(cx.saturating_sub(11), cy, cx + 11, cy + 5, color, px1, fill);
}

fn draw_cloud_shape_small(image: &mut epd::paint::Image, cx: u16, cy: u16, color: epd::paint::Color) {
    let px1 = epd::paint::DotPixel::DotPixel1x1;
    let fill = epd::paint::DrawFill::DrawFillFull;
    image.draw_circle(cx.saturating_sub(4), cy.saturating_sub(1), 3, color, px1, fill);
    image.draw_circle(cx + 1, cy.saturating_sub(3), 4, color, px1, fill);
    image.draw_rectangle(cx.saturating_sub(7), cy + 1, cx + 5, cy + 3, color, px1, fill);
}

fn draw_weather_icon(
    image: &mut epd::paint::Image,
    x: u16,
    y: u16,
    code: u32,
    color: epd::paint::Color,
    size: IconSize,
) {
    let px1 = epd::paint::DotPixel::DotPixel1x1;
    let px2 = epd::paint::DotPixel::DotPixel2x2;
    let solid = epd::paint::LineStyle::LineStyleSolid;
    let fill = epd::paint::DrawFill::DrawFillFull;

    let (cx, cy) = match size {
        IconSize::Small => (x + 10, y + 10),
        IconSize::Large => (x + 14, y + 14),
    };

    match code {
        0 => {
            let (r, ray_s, ray_l, diag_s, diag_l, ray_px) = match size {
                IconSize::Small => (3u16, 5i16, 7i16, 4i16, 5i16, px1),
                IconSize::Large => (5u16, 8i16, 11i16, 6i16, 8i16, px2),
            };
            image.draw_circle(cx, cy, r, color, px1, fill);
            for &(x1, y1, x2, y2) in &[
                (0i16, -ray_s, 0i16, -ray_l),
                (0, ray_s, 0, ray_l),
                (-ray_s, 0, -ray_l, 0),
                (ray_s, 0, ray_l, 0),
                (-diag_s, -diag_s, -diag_l, -diag_l),
                (diag_s, -diag_s, diag_l, -diag_l),
                (-diag_s, diag_s, -diag_l, diag_l),
                (diag_s, diag_s, diag_l, diag_l),
            ] {
                image.draw_line(
                    (cx as i16 + x1) as u16, (cy as i16 + y1) as u16,
                    (cx as i16 + x2) as u16, (cy as i16 + y2) as u16,
                    color, ray_px, solid,
                );
            }
        }
        1..=3 => match size {
            IconSize::Small => draw_cloud_shape_small(image, cx, cy, color),
            IconSize::Large => draw_cloud_shape(image, cx, cy, color),
        },
        45 | 48 => {
            let (extent, spacing, y_off, line_px) = match size {
                IconSize::Small => (7u16, 4u16, 4u16, px1),
                IconSize::Large => (10u16, 5u16, 5u16, px2),
            };
            for i in 0..3u16 {
                image.draw_line(
                    cx.saturating_sub(extent), cy.saturating_sub(y_off) + i * spacing,
                    cx + extent, cy.saturating_sub(y_off) + i * spacing,
                    color, line_px, solid,
                );
            }
        }
        51..=57 | 61..=67 | 80..=82 => {
            let (cloud_dy, drops, x_start, x_step, dy, slant) = match size {
                IconSize::Small => (3u16, 3u16, -4i16, 4u16, 3u16, 1u16),
                IconSize::Large => (5u16, 4u16, -6i16, 4u16, 3u16, 2u16),
            };
            match size {
                IconSize::Small => draw_cloud_shape_small(image, cx, cy.saturating_sub(cloud_dy), color),
                IconSize::Large => draw_cloud_shape(image, cx, cy.saturating_sub(cloud_dy), color),
            }
            for i in 0..drops {
                let rx = (cx as i16 + x_start + (i * x_step) as i16) as u16;
                image.draw_line(
                    rx, cy + dy,
                    rx.saturating_sub(slant), cy + dy + (cloud_dy - 1),
                    color, px1, solid,
                );
            }
        }
        71..=77 | 85 | 86 => {
            let (cloud_dy, dots, x_start, x_step, dot_dy) = match size {
                IconSize::Small => (3u16, 3u16, -4i16, 4u16, 6u16),
                IconSize::Large => (5u16, 4u16, -6i16, 4u16, 7u16),
            };
            match size {
                IconSize::Small => draw_cloud_shape_small(image, cx, cy.saturating_sub(cloud_dy), color),
                IconSize::Large => draw_cloud_shape(image, cx, cy.saturating_sub(cloud_dy), color),
            }
            for i in 0..dots {
                let dx = (cx as i16 + x_start + (i * x_step) as i16) as u16;
                image.draw_circle(dx, cy + dot_dy, 1, color, px1, fill);
            }
        }
        95..=99 => {
            let (cloud_dy, seg1, seg2) = match size {
                IconSize::Small => (4u16, (1i16, 2i16, -1i16, 5i16), (-1i16, 5i16, 2i16, 8i16)),
                IconSize::Large => (6u16, (2i16, 2i16, -2i16, 6i16), (-2i16, 6i16, 3i16, 11i16)),
            };
            match size {
                IconSize::Small => draw_cloud_shape_small(image, cx, cy.saturating_sub(cloud_dy), color),
                IconSize::Large => draw_cloud_shape(image, cx, cy.saturating_sub(cloud_dy), color),
            }
            image.draw_line(
                (cx as i16 + seg1.0) as u16, (cy as i16 + seg1.1) as u16,
                (cx as i16 + seg1.2) as u16, (cy as i16 + seg1.3) as u16,
                color, px2, solid,
            );
            image.draw_line(
                (cx as i16 + seg2.0) as u16, (cy as i16 + seg2.1) as u16,
                (cx as i16 + seg2.2) as u16, (cy as i16 + seg2.3) as u16,
                color, px2, solid,
            );
        }
        _ => {}
    }
}
