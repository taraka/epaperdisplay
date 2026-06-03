use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use ical;
use std::collections::HashMap;
use std::ops::Sub;

const LOOKAHEAD_WEEKS: i64 = 8;

#[derive(PartialEq, Clone)]
enum Repeat {
    None,
    Yearly,
    Weekly(u32),   // interval in weeks
    Monthly(u32),  // interval in months
}

pub struct Event {
    pub name: String,
    pub location: Option<String>,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub all_day: bool,
    pub is_recurring: bool,
    repeat: Repeat,
}

pub fn fetch_data() -> Result<Vec<Event>, String> {
    log::info!("Fetching calendar");

    let url = std::env::var("ICALADDR")
        .map_err(|_| "ICALADDR environment variable not set".to_string())?;

    let body = reqwest::blocking::get(&url)
        .map_err(|e| format!("Calendar request failed: {}", e))?
        .text()
        .map_err(|e| format!("Calendar response unreadable: {}", e))?;

    let cal = match ical::IcalParser::new(body.as_bytes()).next() {
        Some(Ok(c)) => c,
        Some(Err(e)) => return Err(format!("Calendar parse failed: {}", e)),
        None => return Err("Calendar response was empty".to_string()),
    };

    let mut output = Vec::new();

    for e in cal.events {
        let mut props = HashMap::new();
        for p in e.properties {
            if p.value.is_some() {
                props.insert(p.name, p.value.unwrap());
            }
        }

        if props.contains_key("SUMMARY")
            && props.contains_key("DTEND")
            && props.contains_key("DTSTART")
        {
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
                location: props.get("LOCATION").cloned(),
                start,
                end,
                all_day,
                is_recurring: false,
                repeat,
            });
        }
    }

    let now = Utc::now();
    let today_start = now.sub(Duration::seconds(now.timestamp() % 86400));
    let lookahead = today_start + Duration::weeks(LOOKAHEAD_WEEKS);

    let mut output = output
        .into_iter()
        .filter(|e| e.start >= today_start || e.repeat != Repeat::None)
        .flat_map(|e| match e.repeat {
            Repeat::None => vec![e],
            Repeat::Yearly => vec![Event {
                start: find_next_yearly_instance(&e.start, today_start),
                end: find_next_yearly_instance(&e.end, today_start),
                name: e.name,
                location: e.location,
                all_day: e.all_day,
                is_recurring: false,
                repeat: Repeat::None,
            }],
            Repeat::Weekly(interval) => expand_recurring(e, today_start, lookahead, Duration::weeks(interval as i64)),
            Repeat::Monthly(interval) => expand_recurring_monthly(e, today_start, lookahead, interval),
        })
        .collect::<Vec<Event>>();

    output.sort_by(|a, b| a.start.cmp(&b.start).then(a.name.cmp(&b.name)));

    log::info!("Loaded {} upcoming event(s)", output.len());
    Ok(output)
}

fn expand_recurring(
    e: Event,
    today_start: DateTime<Utc>,
    lookahead: DateTime<Utc>,
    step: Duration,
) -> Vec<Event> {
    let duration = e.end - e.start;
    let mut dt = e.start;
    while dt < today_start {
        dt = dt + step;
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
            repeat: Repeat::None,
        });
        dt = dt + step;
    }
    instances
}

fn expand_recurring_monthly(
    e: Event,
    today_start: DateTime<Utc>,
    lookahead: DateTime<Utc>,
    interval: u32,
) -> Vec<Event> {
    let duration = e.end - e.start;
    let mut dt = e.start;
    while dt < today_start {
        for _ in 0..interval { dt = add_one_month(dt); }
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
            repeat: Repeat::None,
        });
        for _ in 0..interval { dt = add_one_month(dt); }
    }
    instances
}

fn add_one_month(dt: DateTime<Utc>) -> DateTime<Utc> {
    let (year, month) = if dt.month() == 12 {
        (dt.year() + 1, 1)
    } else {
        (dt.year(), dt.month() + 1)
    };
    dt.with_year(year)
        .and_then(|d| d.with_month(month))
        .unwrap_or_else(|| dt + Duration::days(28))
}

fn find_next_yearly_instance(dt: &DateTime<Utc>, today_start: DateTime<Utc>) -> DateTime<Utc> {
    let mut mydt = *dt;
    while mydt < today_start {
        mydt = mydt.with_year(mydt.year() + 1).unwrap();
    }
    mydt
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

fn parse_interval(rule: &str) -> u32 {
    rule.split(';')
        .find_map(|part| part.strip_prefix("INTERVAL="))
        .and_then(|v| v.parse().ok())
        .unwrap_or(1)
}

fn get_repeat(rrule: Option<&String>) -> Repeat {
    match rrule {
        Some(rule) => {
            if repeat_expired(rule) {
                return Repeat::None;
            }
            if rule.contains("FREQ=YEARLY") {
                Repeat::Yearly
            } else if rule.contains("FREQ=WEEKLY") {
                Repeat::Weekly(parse_interval(rule))
            } else if rule.contains("FREQ=MONTHLY") {
                Repeat::Monthly(parse_interval(rule))
            } else {
                log::debug!("Unsupported RRULE, ignoring: {}", rule);
                Repeat::None
            }
        }
        None => Repeat::None,
    }
}

fn unpack_time_stamp(input: Option<&String>) -> Option<(DateTime<Utc>, bool)> {
    const FORMAT: &str = "%Y%m%dT%H%M%SZ%z";
    let input_string = input?;
    let result = match DateTime::parse_from_str(&format!("{}{}", input_string, "+0000"), FORMAT) {
        Ok(d) => (d.with_timezone(&Utc), false),
        Err(_) => match DateTime::parse_from_str(&format!("{}Z+0000", input_string), FORMAT) {
            Ok(d1) => (d1.with_timezone(&Utc), false),
            Err(_) => match DateTime::parse_from_str(
                &format!("{}T000000Z+0000", input_string),
                FORMAT,
            ) {
                Ok(d2) => (d2.with_timezone(&Utc), true),
                Err(e) => {
                    log::warn!("Could not parse timestamp {:?}: {}", input_string, e);
                    return None;
                }
            },
        },
    };
    Some(result)
}
