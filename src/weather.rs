use chrono::NaiveDate;
use serde::Deserialize;

pub struct WeatherData {
    pub temperature: i32,
    pub condition: &'static str,
    pub weathercode: u32,
    pub forecast: Vec<DailyForecast>,
    pub sunrise: Option<String>,
    pub sunset: Option<String>,
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
    current: CurrentWeather,
    daily: Option<DailyData>,
}

#[derive(Deserialize)]
struct DailyData {
    time: Vec<String>,
    weathercode: Vec<u32>,
    temperature_2m_max: Vec<f64>,
    sunrise: Vec<String>,
    sunset: Vec<String>,
}

#[derive(Deserialize)]
struct CurrentWeather {
    temperature_2m: f64,
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
    let resp = reqwest::blocking::get(&geo_url);
    let loc = match resp {
        Err(e) => { log::warn!("Geocoding request failed: {}", e); return None; }
        Ok(r) => match r.json::<GeoResponse>() {
            Err(e) => { log::warn!("Geocoding parse failed: {}", e); return None; }
            Ok(g) => g.results?.into_iter().next()?,
        }
    };
    Some((loc.latitude, loc.longitude))
}

pub fn fetch_weather(lat: f64, lon: f64) -> Option<WeatherData> {
    let weather_url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,weathercode&daily=weathercode,temperature_2m_max,sunrise,sunset&timezone=Europe%2FLondon&forecast_days=14",
        lat, lon
    );
    let body = match reqwest::blocking::get(&weather_url) {
        Err(e) => { log::warn!("Weather request failed: {}", e); return None; }
        Ok(r) => match r.text() {
            Err(e) => { log::warn!("Weather response unreadable: {}", e); return None; }
            Ok(t) => t,
        }
    };
    let resp = match serde_json::from_str::<WeatherResponse>(&body) {
        Err(e) => { log::warn!("Weather parse failed: {}\nBody: {}", e, &body[..body.len().min(200)]); return None; }
        Ok(r) => r,
    };

    let cw = resp.current;

    let today = NaiveDate::from(chrono::Local::now().date_naive());

    let (forecast, sunrise, sunset) = match resp.daily {
        None => (Vec::new(), None, None),
        Some(d) => {
            let today_idx = d.time.iter().position(|t| {
                NaiveDate::parse_from_str(t, "%Y-%m-%d").ok().as_ref() == Some(&today)
            });
            let sunrise = today_idx
                .and_then(|i| d.sunrise.get(i))
                .and_then(|s| s.split('T').nth(1))
                .map(|t| t[..5].to_string());
            let sunset = today_idx
                .and_then(|i| d.sunset.get(i))
                .and_then(|s| s.split('T').nth(1))
                .map(|t| t[..5].to_string());
            let items = d.time.iter()
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
                .collect();
            (items, sunrise, sunset)
        }
    };

    Some(WeatherData {
        temperature: cw.temperature_2m.round() as i32,
        condition: wmo_condition(cw.weathercode),
        weathercode: cw.weathercode,
        forecast,
        sunrise,
        sunset,
    })
}
