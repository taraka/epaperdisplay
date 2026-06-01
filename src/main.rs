mod epd;
mod calendar;
mod weather;
mod render;

use epd::display::Display;
use chan::chan_select;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();

    log::info!("e-Paper init and clear");
    let mut display = Display::init();
    display.clear();

    let town = std::env::var("TOWN").ok();
    let cal_url = std::env::var("ICALADDR").ok();
    match &town {
        Some(t) => log::info!("Weather town: {}", t),
        None    => log::warn!("TOWN not set, weather disabled"),
    }
    match &cal_url {
        Some(_) => log::info!("Calendar URL: configured"),
        None    => log::error!("ICALADDR not set — calendar will be empty"),
    }

    let location = town.as_deref().and_then(|_| weather::geocode_town());
    if location.is_none() && town.is_some() {
        log::warn!("Could not geocode town, weather disabled");
    }

    let (mut cal, mut wx) = refresh(&mut display, location);

    let fetch_tick = chan::tick_ms(5 * 60 * 1000);
    let display_tick = chan::tick_ms(epd::display::UPDATE_RATE);
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
        log::warn!("Weather fetch failed");
    }
    render::draw_cal(display, &cal, wx.as_ref());
    (cal, wx)
}
