mod epd;
mod calendar;
mod weather;
mod render;

use epd::display::d7in5_v2::Display;
use chan::chan_select;

fn main() {
    println!("e-Paper Init and Clear...");
    let mut display = Display::init();
    display.clear();

    let location = weather::geocode_town();
    if location.is_none() && std::env::var("TOWN").is_ok() {
        println!("Warning: could not geocode TOWN, weather disabled");
    }

    let (mut cal, mut wx) = refresh(&mut display, location);

    let fetch_tick = chan::tick_ms(5 * 60 * 1000);
    let display_tick = chan::tick_ms(Display::update_rate());
    loop {
        chan_select! {
            display_tick.recv() => {
                render::draw_cal(&mut display, &cal, wx.as_ref());
            },
            fetch_tick.recv() => {
                let (new_cal, new_wx) = refresh(&mut display, location);
                cal = new_cal;
                wx = new_wx;
            }
        }
    }
}

fn refresh(
    display: &mut Display,
    location: Option<(f64, f64)>,
) -> (Vec<calendar::Event>, Option<weather::WeatherData>) {
    let cal = calendar::fetch_data();
    let wx = location.and_then(|(lat, lon)| weather::fetch_weather(lat, lon));
    if wx.is_none() && location.is_some() {
        println!("Warning: weather fetch failed");
    }
    render::draw_cal(display, &cal, wx.as_ref());
    (cal, wx)
}
