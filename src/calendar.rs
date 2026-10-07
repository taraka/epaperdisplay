use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use ical;
use std::collections::HashMap;

const LOOKAHEAD_WEEKS: i64 = 8;

#[derive(PartialEq, Clone)]
enum Repeat {
    None,
    Yearly,
    Weekly(u32),   // interval in weeks
    Monthly(u32),  // interval in months
}

#[derive(PartialEq)]
pub struct Event {
    pub name: String,
    pub location: Option<String>,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub all_day: bool,
    pub is_recurring: bool,
    repeat: Repeat,
    uid: String,
}

pub fn fetch_data() -> Result<Vec<Event>, String> {
    log::info!("Fetching calendar");

    let url = std::env::var("ICALADDR")
        .map_err(|_| "ICALADDR environment variable not set".to_string())?;

    let body = reqwest::blocking::get(&url)
        .map_err(|e| format!("Calendar request failed: {}", e))?
        .text()
        .map_err(|e| format!("Calendar response unreadable: {}", e))?;

    parse_calendar(&body)
}

fn parse_calendar(body: &str) -> Result<Vec<Event>, String> {
    let cal = match ical::IcalParser::new(body.as_bytes()).next() {
        Some(Ok(c)) => c,
        Some(Err(e)) => return Err(format!("Calendar parse failed: {}", e)),
        None => return Err("Calendar response was empty".to_string()),
    };

    // A deleted single instance of a recurring event shows up as either an
    // EXDATE on the master VEVENT, or a separate VEVENT with the same UID
    // carrying a RECURRENCE-ID (the original occurrence time) and
    // STATUS:CANCELLED. Collect both into a per-UID set of occurrence times
    // to suppress from the generated recurrence below.
    let mut excluded: HashMap<String, Vec<DateTime<Utc>>> = HashMap::new();
    for e in &cal.events {
        let mut uid: Option<String> = None;
        let mut recurrence_id: Option<String> = None;
        let mut exdates: Vec<String> = Vec::new();

        for p in &e.properties {
            let value = match &p.value {
                Some(v) => v,
                None => continue,
            };
            match p.name.as_str() {
                "UID" => uid = Some(value.clone()),
                "RECURRENCE-ID" => recurrence_id = Some(value.clone()),
                "EXDATE" => exdates.push(value.clone()),
                _ => {}
            }
        }

        let uid = match uid {
            Some(u) => u,
            None => continue,
        };

        if let Some(rid) = recurrence_id {
            if let Some((dt, _)) = unpack_time_stamp(Some(&rid)) {
                excluded.entry(uid.clone()).or_insert_with(Vec::new).push(dt);
            }
        }
        for exdate in &exdates {
            for part in exdate.split(',') {
                if let Some((dt, _)) = unpack_time_stamp(Some(&part.to_string())) {
                    excluded.entry(uid.clone()).or_insert_with(Vec::new).push(dt);
                }
            }
        }
    }

    let mut output = Vec::new();

    for e in cal.events {
        let mut props = HashMap::new();
        for p in e.properties {
            if p.value.is_some() {
                props.insert(p.name, p.value.unwrap());
            }
        }

        if let Some(status) = props.get("STATUS") {
            if status.eq_ignore_ascii_case("CANCELLED") {
                log::debug!("Skipping cancelled event {:?}", props.get("SUMMARY"));
                continue;
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

            log::debug!(
                "Parsed event {:?}: start={} end={} all_day={}",
                props.get("SUMMARY"), start, end, all_day
            );

            output.push(Event {
                name: props.get("SUMMARY").unwrap().clone(),
                location: props.get("LOCATION").cloned(),
                start,
                end,
                all_day,
                is_recurring: false,
                repeat,
                uid: props.get("UID").cloned().unwrap_or_default(),
            });
        } else if let Some(summary) = props.get("SUMMARY") {
            log::debug!("Skipping event {:?} — missing DTSTART/DTEND, has keys: {:?}", summary, props.keys().collect::<Vec<_>>());
        }
    }

    let now = Utc::now();
    // `now.timestamp() % 86400` truncates to whole seconds, but `now` itself
    // still carries sub-second precision, so subtracting that offset from `now`
    // leaves a few hundred milliseconds of drift past midnight instead of an
    // exact boundary — which made an event starting at exactly 00:00:00.000
    // compare as "before today" and get filtered out. Truncate via the date
    // instead so today_start is always an exact midnight.
    let today_start = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
    let lookahead = today_start + Duration::weeks(LOOKAHEAD_WEEKS);
    log::debug!("now={} today_start={}", now, today_start);

    let mut output = output
        .into_iter()
        .filter(|e| {
            let keep = e.start >= today_start || e.repeat != Repeat::None;
            if !keep {
                log::debug!("Filtering out {:?}: start={} < today_start={}", e.name, e.start, today_start);
            }
            keep
        })
        .flat_map(|e| {
            let ex = excluded.get(&e.uid).map(|v| v.as_slice()).unwrap_or(&[]);
            match e.repeat {
                Repeat::None => vec![e],
                Repeat::Yearly => {
                    let start = find_next_yearly_instance(&e.start, today_start);
                    if ex.contains(&start) {
                        log::debug!("Skipping deleted instance of {:?} at {}", e.name, start);
                        vec![]
                    } else {
                        vec![Event {
                            start,
                            end: find_next_yearly_instance(&e.end, today_start),
                            name: e.name,
                            location: e.location,
                            all_day: e.all_day,
                            is_recurring: false,
                            repeat: Repeat::None,
                            uid: e.uid,
                        }]
                    }
                }
                Repeat::Weekly(interval) => expand_recurring(e, today_start, lookahead, Duration::weeks(interval as i64), ex),
                Repeat::Monthly(interval) => expand_recurring_monthly(e, today_start, lookahead, interval, ex),
            }
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
    excluded: &[DateTime<Utc>],
) -> Vec<Event> {
    let duration = e.end - e.start;
    let mut dt = e.start;
    while dt < today_start {
        dt = dt + step;
    }
    let mut instances = Vec::new();
    while dt <= lookahead {
        if excluded.contains(&dt) {
            log::debug!("Skipping deleted instance of {:?} at {}", e.name, dt);
        } else {
            instances.push(Event {
                name: e.name.clone(),
                location: e.location.clone(),
                start: dt,
                end: dt + duration,
                all_day: e.all_day,
                is_recurring: true,
                repeat: Repeat::None,
                uid: e.uid.clone(),
            });
        }
        dt = dt + step;
    }
    instances
}

fn expand_recurring_monthly(
    e: Event,
    today_start: DateTime<Utc>,
    lookahead: DateTime<Utc>,
    interval: u32,
    excluded: &[DateTime<Utc>],
) -> Vec<Event> {
    let duration = e.end - e.start;
    let mut dt = e.start;
    while dt < today_start {
        for _ in 0..interval { dt = add_one_month(dt); }
    }
    let mut instances = Vec::new();
    while dt <= lookahead {
        if excluded.contains(&dt) {
            log::debug!("Skipping deleted instance of {:?} at {}", e.name, dt);
        } else {
            instances.push(Event {
                name: e.name.clone(),
                location: e.location.clone(),
                start: dt,
                end: dt + duration,
                all_day: e.all_day,
                is_recurring: true,
                repeat: Repeat::None,
                uid: e.uid.clone(),
            });
        }
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
