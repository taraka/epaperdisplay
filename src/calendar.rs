use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use ical;
use std::collections::HashMap;
use std::ops::Sub;

#[derive(PartialEq, Clone)]
enum Repeat {
    NONE,
    YEARLY,
    WEEKLY,
    MONTHLY,
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
    let lookahead = today_start + Duration::weeks(8);

    let mut output = output
        .into_iter()
        .filter(|e| e.start >= today_start || e.repeat != Repeat::NONE)
        .flat_map(|e| match e.repeat {
            Repeat::NONE => vec![e],
            Repeat::YEARLY => vec![Event {
                start: find_next_yearly_instance(&e.start),
                end: find_next_yearly_instance(&e.end),
                name: e.name,
                location: e.location,
                all_day: e.all_day,
                is_recurring: false,
                repeat: Repeat::NONE,
            }],
            Repeat::WEEKLY => expand_recurring(e, today_start, lookahead, Duration::weeks(1)),
            Repeat::MONTHLY => expand_recurring_monthly(e, today_start, lookahead),
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
            repeat: Repeat::NONE,
        });
        dt = dt + step;
    }
    instances
}

fn expand_recurring_monthly(
    e: Event,
    today_start: DateTime<Utc>,
    lookahead: DateTime<Utc>,
) -> Vec<Event> {
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

fn find_next_yearly_instance(dt: &DateTime<Utc>) -> DateTime<Utc> {
    let now = Utc::now();
    let today_start = now.sub(Duration::seconds(now.timestamp() % 86400));
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
                log::debug!("Unsupported RRULE, ignoring: {}", rule);
                Repeat::NONE
            }
        }
        None => Repeat::NONE,
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
