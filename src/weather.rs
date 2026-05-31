use chrono::NaiveDate;
use serde::Deserialize;

pub struct WeatherData {
    pub temperature: i32,
    pub condition: &'static str,
    pub weathercode: u32,
    pub forecast: Vec<DailyForecast>,
}

pub struct DailyForecast {
    pub date: NaiveDate,
    pub temp_max: i32,
    pub weathercode: u32,
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

pub fn wmo_condition(code: u32) -> &'static str {
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

pub fn geocode_town() -> Option<(f64, f64)> {
    let town = std::env::var("TOWN").ok()?;
    let geo_url = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1",
        town
    );
    let loc = reqwest::blocking::get(&geo_url)
        .ok()?
        .json::<GeoResponse>()
        .ok()?
        .results?
        .into_iter()
        .next()?;
    Some((loc.latitude, loc.longitude))
}

pub fn fetch_weather(lat: f64, lon: f64) -> Option<WeatherData> {
    let weather_url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current_weather=true&daily=weathercode,temperature_2m_max&timezone=UTC&forecast_days=14",
        lat, lon
    );
    let resp = reqwest::blocking::get(&weather_url)
        .ok()?
        .json::<WeatherResponse>()
        .ok()?;

    let cw = resp.current_weather;

    let forecast = resp
        .daily
        .map(|d| {
            d.time
                .iter()
                .zip(d.weathercode.iter())
                .zip(d.temperature_2m_max.iter())
                .filter_map(|((date_str, &code), &temp)| {
                    NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                        .ok()
                        .map(|date| DailyForecast {
                            date,
                            temp_max: temp.round() as i32,
                            weathercode: code,
                        })
                })
                .collect()
        })
        .unwrap_or_default();

    Some(WeatherData {
        temperature: cw.temperature.round() as i32,
        condition: wmo_condition(cw.weathercode),
        weathercode: cw.weathercode,
        forecast,
    })
}
