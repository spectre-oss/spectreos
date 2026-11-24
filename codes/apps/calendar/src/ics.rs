use crate::model::{Event, EventId, local_zone};
use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Tz;
use ical::property::Property;
use std::{
    fmt::Write,
    fs,
    io::{self, Read},
    path::Path,
};

fn escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace("\r\n", "\n")
        .replace(['\r', '\n'], "\\n")
        .replace(',', "\\,")
        .replace(';', "\\;")
}

fn unescape(value: &str) -> String {
    let mut chars = value.chars();
    let mut text = String::new();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('n' | 'N') => text.push('\n'),
                Some(c) => text.push(c),
                None => text.push('\\'),
            }
        } else {
            text.push(ch);
        }
    }
    text
}

fn property<'a>(props: &'a [Property], key: &str) -> Option<&'a Property> {
    props
        .iter()
        .find(|property| property.name.eq_ignore_ascii_case(key))
}
fn value<'a>(props: &'a [Property], key: &str) -> Option<&'a str> {
    property(props, key)?.value.as_deref()
}

fn moment(prop: &Property, value: &str, default: Tz) -> Option<(NaiveDateTime, Tz, bool)> {
    if let Ok(date) = NaiveDate::parse_from_str(value, "%Y%m%d") {
        return Some((date.and_hms_opt(0, 0, 0)?, chrono_tz::UTC, true));
    }
    let (value, zone) = if let Some(utc) = value.strip_suffix('Z') {
        (utc, chrono_tz::UTC)
    } else {
        let zone = prop.params.as_ref().and_then(|params| {
            params
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case("TZID"))
                .and_then(|(_, items)| items.first())
        });
        (
            value,
            match zone {
                Some(zone) => zone.parse().ok()?,
                None => default,
            },
        )
    };
    let date = NaiveDateTime::parse_from_str(value, "%Y%m%dT%H%M%S").ok()?;
    Some((date, zone, false))
}

fn as_utc(moment: (NaiveDateTime, Tz, bool)) -> Option<DateTime<Utc>> {
    moment
        .1
        .from_local_datetime(&moment.0)
        .earliest()
        .map(|item| item.to_utc())
}

fn duration(value: &str) -> Option<Duration> {
    let (factor, value) = if let Some(value) = value.strip_prefix('-') {
        (-1_i64, value)
    } else {
        (1, value.strip_prefix('+').unwrap_or(value))
    };
    let value = value.strip_prefix('P')?;
    let mut total = 0_i64;
    let mut number: Option<i64> = None;
    for ch in value.chars() {
        if let Some(digit) = ch.to_digit(10) {
            number = Some(
                number
                    .unwrap_or(0)
                    .checked_mul(10)?
                    .checked_add(i64::from(digit))?,
            );
        } else if ch != 'T' {
            let scale = match ch {
                'W' => 604800,
                'D' => 86400,
                'H' => 3600,
                'M' => 60,
                'S' => 1,
                _ => return None,
            };
            total = total.checked_add(number.take()?.checked_mul(scale)?)?;
        }
    }
    if number.is_some() {
        return None;
    }
    Duration::try_seconds(total.checked_mul(factor)?)
}

pub(crate) struct Imported {
    pub events: Vec<Event>,
    pub skipped: usize,
}

fn event(props: &[Property], calendar: &str, alarm_triggers: &[&str]) -> Option<Event> {
    if value(props, "STATUS") == Some("CANCELLED") {
        return None;
    }
    let uid = value(props, "UID")
        .map(unescape)
        .filter(|uid| !uid.is_empty())
        .unwrap_or_else(crate::model::id);
    let first = property(props, "DTSTART")?;
    let (start, zone, all_day) = moment(first, first.value.as_deref()?, local_zone())?;
    let start_utc = as_utc((start, zone, all_day))?;
    let end = match property(props, "DTEND") {
        Some(prop) => as_utc(moment(prop, prop.value.as_deref()?, zone)?)?
            .with_timezone(&zone)
            .naive_local(),
        None => {
            let span = value(props, "DURATION")
                .and_then(duration)
                .unwrap_or_else(|| {
                    if all_day {
                        Duration::days(1)
                    } else {
                        Duration::hours(1)
                    }
                });
            start_utc
                .checked_add_signed(span)?
                .with_timezone(&zone)
                .naive_local()
        }
    };
    let dates = |name: &str| -> Vec<DateTime<Utc>> {
        props
            .iter()
            .filter(|prop| prop.name == name)
            .flat_map(|prop| {
                prop.value.iter().flat_map(move |value| {
                    value
                        .split(',')
                        .filter_map(move |date| as_utc(moment(prop, date, zone)?))
                })
            })
            .collect()
    };
    let instance = match property(props, "RECURRENCE-ID") {
        Some(prop) => Some(as_utc(moment(prop, prop.value.as_deref()?, zone)?)?),
        None => None,
    };
    let input_rule = value(props, "RRULE").map(str::to_owned);
    if let Some(rule) = &input_rule {
        format!(
            "DTSTART;TZID={zone}:{}\nRRULE:{rule}",
            start.format("%Y%m%dT%H%M%S")
        )
        .parse::<rrule::RRuleSet>()
        .ok()?;
    }
    let reminders = alarm_triggers
        .iter()
        .filter_map(|trigger| {
            let seconds = duration(trigger)?.num_seconds();
            (seconds <= 0 && seconds >= -2_592_000).then(|| (-seconds / 60) as u32)
        })
        .collect();
    let title = value(props, "SUMMARY")
        .map(unescape)
        .filter(|text| !text.trim().is_empty())
        .unwrap_or_else(|| "Untitled event".into());
    let result = Event {
        id: EventId { uid, instance },
        calendar: calendar.into(),
        title,
        location: value(props, "LOCATION").map(unescape).unwrap_or_default(),
        notes: value(props, "DESCRIPTION")
            .map(unescape)
            .unwrap_or_default(),
        start,
        end,
        zone,
        all_day,
        rule: input_rule,
        added_dates: dates("RDATE"),
        exceptions: dates("EXDATE"),
        reminders,
    };
    result.valid().then_some(result)
}

pub(crate) fn read(path: &Path, calendar: &str) -> io::Result<Imported> {
    const MAX_BYTES: u64 = 12 * 1024 * 1024;
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(io::Error::other("Calendar file is larger than 12 MB"));
    }
    parse(&bytes, calendar)
}

fn parse(bytes: &[u8], calendar: &str) -> io::Result<Imported> {
    let bytes = bytes.strip_prefix(b"\xef\xbb\xbf").unwrap_or(bytes);
    let mut imported = Imported {
        events: vec![],
        skipped: 0,
    };
    let mut count = 0;
    for source in ical::IcalParser::new(bytes) {
        count += 1;
        for item in source
            .map_err(|error| io::Error::other(error.to_string()))?
            .events
        {
            let alarms: Vec<_> = item
                .alarms
                .iter()
                .filter(|alarm| value(&alarm.properties, "ACTION") == Some("DISPLAY"))
                .filter_map(|alarm| {
                    let trigger = property(&alarm.properties, "TRIGGER")?;
                    let end = trigger.params.as_ref().is_some_and(|params| {
                        params.iter().any(|(key, values)| {
                            key == "RELATED" && values.iter().any(|value| value == "END")
                        })
                    });
                    if end { None } else { trigger.value.as_deref() }
                })
                .collect();
            if let Some(event) = event(&item.properties, calendar, &alarms) {
                imported.events.push(event);
            } else {
                imported.skipped += 1;
            }
        }
    }
    if count == 0 {
        return Err(io::Error::other("This is not an iCalendar file"));
    }
    Ok(imported)
}

fn push_folded(output: &mut String, line: &str) {
    // RFC 5545 permits folded content lines. Split on UTF-8 boundaries.
    let mut bytes = 0;
    for ch in line.chars() {
        if bytes + ch.len_utf8() > 75 {
            output.push_str("\r\n ");
            bytes = 1;
        }
        output.push(ch);
        bytes += ch.len_utf8();
    }
    output.push_str("\r\n");
}

pub(crate) fn encode<'a>(events: impl IntoIterator<Item = &'a Event>) -> String {
    let mut text = String::from(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//SpectreOS//Calendar//EN\r\nCALSCALE:GREGORIAN\r\n",
    );
    let stamp = crate::model::now_utc().format("%Y%m%dT%H%M%SZ");
    for event in events {
        let (Some(start), Some(end)) = (event.start_utc(), event.end_utc()) else {
            continue;
        };
        text.push_str("BEGIN:VEVENT\r\n");
        push_folded(&mut text, &format!("UID:{}", escape(&event.id.uid)));
        let _ = write!(text, "DTSTAMP:{stamp}\r\n");
        for (kind, local, utc) in [("DTSTART", event.start, start), ("DTEND", event.end, end)] {
            if event.all_day {
                let _ = write!(text, "{kind};VALUE=DATE:{}\r\n", local.format("%Y%m%d"));
            } else if event.zone == chrono_tz::UTC {
                let _ = write!(text, "{kind}:{}\r\n", utc.format("%Y%m%dT%H%M%SZ"));
            } else {
                let _ = write!(
                    text,
                    "{kind};TZID={}:{}\r\n",
                    event.zone,
                    local.format("%Y%m%dT%H%M%S")
                );
            }
        }
        if let Some(instance) = event.id.instance {
            let _ = if event.all_day {
                write!(
                    text,
                    "RECURRENCE-ID;VALUE=DATE:{}\r\n",
                    instance.format("%Y%m%d")
                )
            } else {
                write!(
                    text,
                    "RECURRENCE-ID:{}\r\n",
                    instance.format("%Y%m%dT%H%M%SZ")
                )
            };
        }
        for (name, value) in [
            ("SUMMARY", &event.title),
            ("LOCATION", &event.location),
            ("DESCRIPTION", &event.notes),
        ] {
            if !value.is_empty() {
                push_folded(&mut text, &format!("{name}:{}", escape(value)));
            }
        }
        if let Some(rule) = &event.rule {
            push_folded(&mut text, &format!("RRULE:{rule}"));
        }
        for (name, dates) in [("RDATE", &event.added_dates), ("EXDATE", &event.exceptions)] {
            if dates.is_empty() {
                continue;
            }
            let format = if event.all_day {
                "%Y%m%d"
            } else {
                "%Y%m%dT%H%M%SZ"
            };
            let values = dates
                .iter()
                .map(|date| date.format(format).to_string())
                .collect::<Vec<_>>()
                .join(",");
            let typ = if event.all_day { ";VALUE=DATE" } else { "" };
            push_folded(&mut text, &format!("{name}{typ}:{values}"));
        }
        for minutes in &event.reminders {
            let _ = write!(
                text,
                "BEGIN:VALARM\r\nACTION:DISPLAY\r\nTRIGGER:-PT{minutes}M\r\n"
            );
            push_folded(&mut text, &format!("DESCRIPTION:{}", escape(&event.title)));
            text.push_str("END:VALARM\r\n");
        }
        text.push_str("END:VEVENT\r\n");
    }
    text.push_str("END:VCALENDAR\r\n");
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recurring_all_day_event_round_trips_unicode_text_and_exclusions() {
        let sample = b"BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:sample\r\nDTSTART;VALUE=DATE:20261128\r\nDTEND;VALUE=DATE:20261129\r\nSUMMARY:Review\\, notes\\nSecond line\r\nRRULE:FREQ=WEEKLY;COUNT=3\r\nEXDATE;VALUE=DATE:20261205\r\nBEGIN:VALARM\r\nACTION:DISPLAY\r\nTRIGGER:-PT15M\r\nEND:VALARM\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
        let mut imported = parse(sample, "personal").unwrap();
        assert_eq!(imported.skipped, 0);
        let mut event = imported.events.pop().unwrap();
        assert_eq!(event.title, "Review, notes\nSecond line");
        assert_eq!(event.reminders, [15]);
        event.notes = "Åℎ☎".repeat(44);
        let output = encode([&event]);
        assert!(output.lines().all(|line| line.len() <= 75));
        let mut data = crate::model::Data::default();
        data.put(
            parse(output.as_bytes(), "personal")
                .unwrap()
                .events
                .remove(0),
        );
        assert_eq!(data.events, [event]);
        let instances = data.instances(
            NaiveDate::from_ymd_opt(2026, 11, 28).unwrap(),
            NaiveDate::from_ymd_opt(2026, 12, 16).unwrap(),
            "",
            true,
        );
        assert_eq!(instances.len(), 2);
        assert_eq!(
            instances[1].display_start().date(),
            NaiveDate::from_ymd_opt(2026, 12, 12).unwrap()
        );
    }
}
