mod epd;
mod calendar;
mod weather;
mod render;

use epd::display::Display;
use render::WeatherStatus;
use chan::chan_select;

struct State {
    cal: Vec<calendar::Event>,
    wx: Option<weather::WeatherData>,
    error: Option<String>,
}

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
    let weather_configured = town.is_some();

    let mut state = fetch_and_draw(&mut display, location, weather_configured);

    let fetch_tick = chan::tick_ms(5 * 60 * 1000);
    let display_tick = chan::tick_ms(epd::display::UPDATE_RATE);
    loop {
        chan_select! {
            display_tick.recv() => {
                redraw(&mut display, &state, location, weather_configured);
            },
            fetch_tick.recv() => {
                state = fetch_and_draw(&mut display, location, weather_configured);
            }
        }
    }
}

fn fetch_and_draw(
    display: &mut Display,
    location: Option<(f64, f64)>,
    weather_configured: bool,
) -> State {
    let (cal, error) = match calendar::fetch_data() {
        Ok(events) => (events, None),
        Err(e) => {
            log::error!("{}", e);
            render::draw_error(display, &e);
            return State { cal: Vec::new(), wx: None, error: Some(e) };
        }
    };

    let wx = location.and_then(|(lat, lon)| weather::fetch_weather(lat, lon));
    if wx.is_none() && location.is_some() {
        log::warn!("Weather fetch failed");
    }

    render::draw_cal(display, &cal, weather_status(location, wx.as_ref(), weather_configured));
    State { cal, wx, error }
}

fn redraw(
    display: &mut Display,
    state: &State,
    location: Option<(f64, f64)>,
    weather_configured: bool,
) {
    if let Some(e) = &state.error {
        render::draw_error(display, e);
    } else {
        render::draw_cal(display, &state.cal, weather_status(location, state.wx.as_ref(), weather_configured));
    }
}

fn weather_status<'a>(
    location: Option<(f64, f64)>,
    wx: Option<&'a weather::WeatherData>,
    weather_configured: bool,
) -> WeatherStatus<'a> {
    match (weather_configured, location, wx) {
        (false, _, _)      => WeatherStatus::Disabled,
        (true, _, Some(w)) => WeatherStatus::Available(w),
        (true, _, None)    => WeatherStatus::Unavailable,
    }
}
