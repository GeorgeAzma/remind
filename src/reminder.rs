use crate::Interval;
use chrono::{DateTime, Datelike, Duration, Local, Months, NaiveDateTime, TimeZone};

pub struct Reminder {
    pub title: String,
    pub interval: Interval,
    pub end_time: DateTime<Local>,
    pub repeats: u32,
    pub skips: u32,
    pub weekdays: u8,
}

pub const SUNDAY: u8 = 0b0000001;
pub const MONDAY: u8 = 0b0000010;
pub const TUESDAY: u8 = 0b0000100;
pub const WEDNESDAY: u8 = 0b0001000;
pub const THURSDAY: u8 = 0b0010000;
pub const FRIDAY: u8 = 0b0100000;
pub const SATURDAY: u8 = 0b1000000;

impl Reminder {
    pub fn weekdays_to_str(&self) -> String {
        let bits = if self.weekdays == 0 {
            !0u8
        } else {
            self.weekdays
        };
        let mut weekdays = String::new();
        const DAY_NAMES: [&str; 7] = ["sun", "mon", "tue", "wed", "thu", "fri", "sat"];
        for (i, day) in DAY_NAMES.into_iter().enumerate() {
            if bits & (1 << i) > 0 {
                weekdays += day;
                weekdays += " ";
            }
        }
        weekdays.pop();
        weekdays
    }

    pub fn serialize(&self) -> String {
        format!(
            "{}⌠{}⌠{}⌠{}⌠{}⌠{}\n",
            self.title,
            self.interval.serialize(),
            self.end_time.format("%y-%m-%d %H:%M:%S"),
            self.repeats,
            self.skips,
            self.weekdays_to_str()
        )
    }

    pub fn deserialize(line: &str) -> Self {
        let data: Vec<&str> = line.split('⌠').collect();
        let weekdays = data[5].split(" ");
        let mut weekday_bits = 0;
        for day in weekdays {
            weekday_bits |= match day {
                "sun" => SUNDAY,
                "mon" => MONDAY,
                "tue" => TUESDAY,
                "wed" => WEDNESDAY,
                "thu" => THURSDAY,
                "fri" => FRIDAY,
                "sat" => SATURDAY,
                _ => {
                    eprintln!("invalid weekday pattern encountered while parsing reminders.txt");
                    0
                }
            }
        }
        Self {
            title: data[0].to_owned(),
            interval: Interval::deserialize(data[1]),
            end_time: Local
                .from_local_datetime(
                    &NaiveDateTime::parse_from_str(data[2], "%y-%m-%d %H:%M:%S")
                        .unwrap_or_default(),
                )
                .single()
                .unwrap_or_default(),
            repeats: data[3].parse().unwrap_or_default(),
            skips: data[4].parse().unwrap_or_default(),
            weekdays: weekday_bits,
        }
    }

    fn weekdays_match_end_weekday(&self) -> bool {
        let mut weekdays = self.weekdays;
        if weekdays == 0 {
            weekdays = u8::MAX;
        }
        use chrono::Weekday;
        let matches = match self.end_time.weekday() {
            Weekday::Mon => weekdays & MONDAY,
            Weekday::Tue => weekdays & TUESDAY,
            Weekday::Wed => weekdays & WEDNESDAY,
            Weekday::Thu => weekdays & THURSDAY,
            Weekday::Fri => weekdays & FRIDAY,
            Weekday::Sat => weekdays & SATURDAY,
            Weekday::Sun => weekdays & SUNDAY,
        };
        matches > 0
    }

    // updates repeating reminder's end time so that remind time is not up anymore
    // returns (updated, should_remove)
    pub fn update(&mut self) -> (bool, bool) {
        let now = Local::now();
        let mut updated = false;
        let always_repeats = self.repeats == 0;
        while self.end_time <= now {
            updated = true;
            if !always_repeats {
                self.repeats -= 1;
                if self.repeats == 0 {
                    return (updated, true);
                }
            }
            self.end_time = self
                .end_time
                .with_year(self.end_time.year() + self.interval.years as i32)
                .unwrap();
            self.end_time = self
                .end_time
                .checked_add_months(Months::new(self.interval.months))
                .unwrap();
            self.end_time += Duration::days(self.interval.days as i64);
            self.end_time += Duration::hours(self.interval.hours as i64);
            self.end_time += Duration::minutes(self.interval.mins as i64);
            self.end_time += Duration::seconds(self.interval.secs as i64);
        }
        while !self.weekdays_match_end_weekday() {
            self.end_time += Duration::days(1);
        }
        let should_remove = !always_repeats && self.repeats == 0;
        (updated, should_remove)
    }
}

impl std::fmt::Display for Reminder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let now = Local::now();
        let end = self.end_time;
        let off = end - now;
        let years = off.num_days() / 365;
        let months = (off.num_days() - years * 365) / 30;
        let weeks = (off.num_days() - months * 30) / 7;
        let days = off.num_days() % 7;
        let hours = off.num_hours() % 24;
        let mins = off.num_minutes() % 60;
        let secs = off.num_seconds() % 60;
        let mut due_str = String::new();
        let mut fmt = |time: i64, unit: &str| {
            if time > 0 {
                due_str += &format!(" {time}{unit}");
            }
        };
        fmt(years, "y");
        fmt(months, "mo");
        if years == 0 {
            fmt(weeks, "w");
            fmt(days, "d");
            if months == 0 && weeks == 0 {
                fmt(hours, "h");
                if days == 0 {
                    fmt(mins, "m");
                    if hours == 0 {
                        fmt(secs, "s");
                    }
                }
            }
        }
        let due_str = if due_str.is_empty() {
            String::new()
        } else {
            format!(" (in{})", due_str)
        };
        let title = &self.title;
        let weekdays = match self.weekdays {
            0 | 127 | 255 => String::new(),
            _ => format!(" [{}]", self.weekdays_to_str()),
        };
        let repeat = match self.repeats {
            0 => " [repeat]".to_string(),
            1 => " [once]".to_string(),
            n => format!(" [{n} times]"),
        };
        let interval_str = if self.repeats == 0 || self.repeats > 1 {
            match self.interval {
                Interval {
                    secs: 0,
                    mins: 0,
                    hours: 0,
                    days: 0,
                    months: 0,
                    years: 0,
                } => " [never]".to_string(),
                Interval {
                    secs: 0,
                    mins: 0,
                    hours: 1,
                    days: 0,
                    months: 0,
                    years: 0,
                } => " [hourly]".to_string(),
                Interval {
                    secs: 0,
                    mins: 0,
                    hours: 0,
                    days: 1,
                    months: 0,
                    years: 0,
                } => " [daily]".to_string(),
                Interval {
                    secs: 0,
                    mins: 0,
                    hours: 0,
                    days: 7,
                    months: 0,
                    years: 0,
                } => " [weekly]".to_string(),
                Interval {
                    secs: 0,
                    mins: 0,
                    hours: 0,
                    days: 0,
                    months: 1,
                    years: 0,
                } => " [monthly]".to_string(),
                Interval {
                    secs: 0,
                    mins: 0,
                    hours: 0,
                    days: 0,
                    months: 0,
                    years: 1,
                } => " [yearly]".to_string(),

                Interval {
                    secs: s,
                    mins: m,
                    hours: h,
                    days: d,
                    months: mo,
                    years: y,
                } => {
                    let w = d / 7;
                    let d = d % 7;
                    format!(
                        " [every{}{}{}{}{}{}{}]",
                        (y > 0).then_some(format!(" {y}y")).unwrap_or_default(),
                        (mo > 0).then_some(format!(" {mo}mo")).unwrap_or_default(),
                        (w > 0).then_some(format!(" {w}w")).unwrap_or_default(),
                        (d > 0).then_some(format!(" {d}d")).unwrap_or_default(),
                        (h > 0).then_some(format!(" {h}h")).unwrap_or_default(),
                        (m > 0).then_some(format!(" {m}m")).unwrap_or_default(),
                        (s > 0).then_some(format!(" {s}s")).unwrap_or_default(),
                    )
                }
            }
        } else {
            String::new()
        };
        let skip = if self.skips == 0 {
            String::new()
        } else if self.skips == 1 {
            " [skip]".to_string()
        } else {
            format!(" [skip {} times]", self.skips)
        };
        let mut end = self.end_time.format("%y-%m-%d %H:%M:%S").to_string();
        if end.ends_with(":00") {
            end = end[..end.len() - 3].to_string();
        }
        f.write_fmt(format_args!(
            "\"{title}\"{skip}{repeat}{weekdays}{interval_str} [{end}]{due_str}"
        ))
    }
}
