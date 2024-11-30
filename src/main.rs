use chrono::{Datelike, Duration, Local, Months, NaiveTime, Timelike};
mod time_unit;
use time_unit::*;
mod reminder;
use reminder::*;
mod reminder_file;
use reminder_file::*;
mod interval;
use interval::*;

// TODO: undo

#[derive(Debug, Clone, PartialEq)]
enum Arg {
    Title(String),
    Number(u32),
    TimeUnit(TimeUnit),
    Remove,
    Repeat(u32),
    WeekDay(u8),
    Time(u32, u32, u32), // hour, min, sec
    Month(u32),
    Skip(u32),
    Undo,
    Clear,
    List,
    Help,
}

fn print_help() {
    let help_str = r#"
Examples:
    Add Reminders:
        $ remind 1d "code tomorrow"
        $ remind minute "egg ready" repeat 4
        $ remind 12:30:15 feb 28 2029
        $ remind monday fri "study"
        $ remind weekly work "go to work" # 5 days a week, at current time
        $ remind weekend "rest" rep 8
        $ remind skip 2 "rest" # skip 2 weekends cause boss sucks
        $ remind daily 11am workout
        $ remind undo

    List Reminders:
        $ remind list

    Remove Reminders (fuzzy):
        $ remind rm "some long name..."
        $ remind clear

Aliases:
    Time:
        - s[econds] | sc | secs | scnds | secndo | sencod | secodn | secnod
        - m[inutes] | mn | mns | mintue | minteu | mitneu
        - h[ours] | hr | hrs | hs | horus
        - d[ays] | ds
        - w[eeks] | wk | wks
        - mo | mont | month | months | mnth | mnths
        - y[ears] | yr | yrs | ys
    
    Weekday:
        - su[nday] | sn | snd
        - mon[day] | md | mn | mnd
        - tu[esday] | tsd
        - we[dnesday] | wd | wdnsd
        - th[ursday] | thrsd
        - fr[iday] | fd | frd
        - mon,thu | mo-th | mo|th | mo+th | mo-th | mo/th | mo/th | mo\th | mo_th
        - work | wrk | business | biz | busy | workweek | work-week | su,mo,tu,we,th,fr,sa
        - weeke[nd] | weeknd | wknd | wkd | break | brk | holiday| rest | week-end | sun,sat | sa+su
    
    Month:
        - january | janu | jan
        - february | febr | feb
        - march | marc | mar
        - april | aprl | apr
        - may
        - june | jun
        - july | jul
        - august | augu | aug
        - september | sept | sep
        - october | octo | oct
        - november | nove | nov
        - december | dece | dec
    
    Repeat:
        - rep[eat] | rp | times
        - repeating | repetetive | loop | looping | infinite | ongoing | recurring | cyclic | series
        - skip | sk | skp | snooze | snz | skip-next | sk-next | skp-next | snooze-next | snz-next
        - hourly | everyhour | every-hour
        - daily | everyday | every-day
        - weekly | everyweek | every-week
        - monthly | everymonth | every-month
        - yearly | everyyear | every-year | annual | annually | anual | anually

    Commands:
        - undo | goback | go-back
        - clear | clean | cls | clr | remove-all | rm-all | del-all | delete-all | erase-all | rmv-all | dlt-all  
        - r[emove] | rm | rmv | de[lete] | dl | dlt | erase | forget | forgt | frgt
        - l[ist] | ls | reminders | all | see | everything
        - h | help | hlp
"#;
    println!("{help_str}");
}

// 32week4 -> (32, week, 4)
fn num_str_num(str: &str) -> (u32, String, u32) {
    let first_idx = str.chars().position(|c| !c.is_ascii_digit()).unwrap_or(0);
    let first_dig = str[..first_idx].parse().unwrap_or(0);
    let last_idx = str[first_idx..]
        .chars()
        .position(|c| c.is_numeric())
        .unwrap_or(str.len() - first_idx)
        + first_idx;
    let last_dig = str[first_idx..]
        .chars()
        .filter(|c| c.is_numeric())
        .collect::<String>()
        .parse::<u32>()
        .unwrap_or(0);
    (first_dig, str[first_idx..last_idx].to_owned(), last_dig)
}

fn tokenize(args: &[String]) -> Vec<Arg> {
    let mut arg_toks = Vec::new();
    for arg in args.iter().skip(1) {
        let arg = arg.as_str();
        let arg_tok: Arg = if let Ok(num) = arg.parse() {
            Arg::Number(num)
        } else {
            let (arg_num1, arg_str, arg_num2) = num_str_num(arg);
            let num = arg_num1.max(arg_num2);
            match arg_str.as_str() {
                "rep" | "repe" | "repea" | "repeat" | "rp" | "times" => Arg::Repeat(num),
                "repeating" | "infinite" | "series" | "recurring" | "loop" | "looping"
                | "cyclic" | "ongoing" | "repetetive" => Arg::Repeat(0),
                "r" | "re" | "rem" | "remo" | "remov" | "remove" | "rm" | "rmv" | "de" | "del"
                | "dele" | "delet" | "delete" | "dl" | "dlt" | "erase" | "forget" | "forgt"
                | "frgt" => Arg::Remove,
                "hourly" | "everyhour" | "every-hour" => {
                    arg_toks.push(Arg::Repeat(0));
                    Arg::TimeUnit(TimeUnit::Hour(num.max(1)))
                }
                "daily" | "everyday" | "every-day" => {
                    arg_toks.push(Arg::Repeat(0));
                    Arg::TimeUnit(TimeUnit::Day(num.max(1)))
                }
                "weekly" | "everyweek" | "every-week" => {
                    arg_toks.push(Arg::Repeat(0));
                    Arg::TimeUnit(TimeUnit::Week(num.max(1)))
                }
                "monthly" | "everymonth" | "every-month" => {
                    arg_toks.push(Arg::Repeat(0));
                    Arg::TimeUnit(TimeUnit::Month(num.max(1)))
                }
                "yearly" | "everyyear" | "every-year" | "annual" | "annually" | "anual"
                | "anually" => {
                    arg_toks.push(Arg::Repeat(0));
                    Arg::TimeUnit(TimeUnit::Year(num.max(1)))
                }
                "weekend" | "weeken" | "weeke" | "weeknd" | "wknd" | "wkd" | "break" | "brk"
                | "holiday" | "rest" | "week-end" | "sun,sat" | "sat,sun" | "sa,su" | "su,sa"
                | "sn,st" | "st,sn" | "sun|sat" | "sat|sun" | "sa|su" | "su|sa" | "sn|st"
                | "st|sn" | "sun+sat" | "sat+sun" | "sa+su" | "su+sa" | "sn+st" | "st+sn"
                | "sun-sat" | "sat-sun" | "sa-su" | "su-sa" | "sn-st" | "st-sn" | "sun_sat"
                | "sat_sun" | "sa_su" | "su_sa" | "sn_st" | "st_sn" => {
                    Arg::WeekDay(SUNDAY | SATURDAY)
                }
                "work" | "wrk" | "business" | "biz" | "busy" | "workweek" | "work-week" => {
                    Arg::WeekDay(MONDAY | TUESDAY | WEDNESDAY | THURSDAY | FRIDAY)
                }
                "l" | "li" | "lis" | "list" | "ls" | "reminders" | "all" | "see" | "everything" => {
                    return vec![Arg::List]
                }
                "clear" | "clean" | "cls" | "clr" | "remove-all" | "rm-all" | "del-all"
                | "delete-all" | "erase-all" | "rmv-all" | "dlt-all" => return vec![Arg::Clear],
                "h" | "help" | "hlp" if num == 0 => return vec![Arg::Help],
                "s" | "se" | "sec" | "seco" | "secon" | "second" | "seconds" | "secs" | "sc"
                | "scnd" | "scnds" | "secndo" | "sencod" | "secodn" | "secnod" => {
                    Arg::TimeUnit(TimeUnit::Second(num))
                }
                "m" | "mi" | "min" | "minu" | "minut" | "minute" | "minutes" | "mins" | "mn"
                | "mnt" | "mnts" | "mintue" | "minteu" | "mitneu" | "mns" => {
                    Arg::TimeUnit(TimeUnit::Minute(num))
                }
                "h" | "ho" | "hou" | "hour" | "hr" | "hrs" | "hours" | "hs" | "horus" => {
                    Arg::TimeUnit(TimeUnit::Hour(num))
                }
                "d" | "da" | "day" | "days" | "ds" => Arg::TimeUnit(TimeUnit::Day(num)),
                "w" | "we" | "wee" | "week" | "weeks" | "wk" | "wks" => {
                    Arg::TimeUnit(TimeUnit::Week(num))
                }
                "mo" | "mont" | "month" | "months" | "mnth" | "mnths" => {
                    Arg::TimeUnit(TimeUnit::Month(num))
                }
                "y" | "ye" | "yea" | "year" | "years" | "yr" | "yrs" | "ys" => {
                    Arg::TimeUnit(TimeUnit::Year(num))
                }
                "su" | "sun" | "sund" | "sunda" | "sunday" | "sn" | "snd" => Arg::WeekDay(SUNDAY),
                "mon" | "mond" | "monda" | "monday" | "md" | "mnd" => Arg::WeekDay(MONDAY),
                "tu" | "tue" | "tues" | "tuesd" | "tuesda" | "tuesday" | "tsd" => {
                    Arg::WeekDay(TUESDAY)
                }
                "wed" | "wedn" | "wedne" | "wednes" | "wednesd" | "wednesda" | "wednesday"
                | "wd" | "wednsd" | "wdnsd" => Arg::WeekDay(WEDNESDAY),
                "th" | "thu" | "thur" | "thurs" | "thursd" | "thursda" | "thursday" | "thrsd" => {
                    Arg::WeekDay(THURSDAY)
                }
                "fr" | "fri" | "frid" | "frida" | "friday" | "fd" | "frd" => Arg::WeekDay(FRIDAY),
                "sa" | "sat" | "satu" | "satur" | "saturd" | "saturda" | "saturday" | "st"
                | "strd" => Arg::WeekDay(SATURDAY),
                "january" | "janu" | "jan" => Arg::Month(0),
                "february" | "febr" | "feb" => Arg::Month(1),
                "march" | "marc" | "mar" => Arg::Month(2),
                "april" | "aprl" | "apr" => Arg::Month(3),
                "may" => Arg::Month(4),
                "june" | "jun" => Arg::Month(5),
                "july" | "jul" => Arg::Month(6),
                "august" | "augu" | "aug" => Arg::Month(7),
                "september" | "sept" | "sep" => Arg::Month(8),
                "october" | "octo" | "oct" => Arg::Month(9),
                "november" | "nove" | "nov" => Arg::Month(10),
                "december" | "dece" | "dec" => Arg::Month(11),
                "skip" | "sk" | "skp" | "snooze" | "snz" | "skip-next" | "sk-next" | "skp-next"
                | "snooze-next" | "snz-next" => Arg::Skip(num),
                "undo" | "goback" | "go-back" => Arg::Undo,
                _ => {
                    let mut arg_str = arg;
                    let pm = arg_str.ends_with("pm");
                    let am = arg_str.ends_with("am");
                    if pm || am || arg_str.contains(':') {
                        if pm || am {
                            arg_str = &arg_str[..arg_str.len() - 2];
                        }
                        if arg_str.ends_with(':') {
                            arg_str = &arg_str[..arg_str.len() - 1];
                        }
                        let mut it = arg_str.split(':');
                        const E: u32 = u32::MAX;
                        let (mut hour, min, sec) = (
                            it.next().and_then(|h| h.parse().ok()).unwrap_or(E),
                            it.next().and_then(|m| m.parse().ok()).unwrap_or(E),
                            it.next().and_then(|s| s.parse().ok()).unwrap_or(E),
                        );
                        if pm && hour != E {
                            hour = (hour + 12) % 24;
                        }
                        let now = Local::now();
                        match (hour, min, sec) {
                            (E, E, E) => Arg::Title(arg.to_owned()),
                            (h, E, E) => Arg::Time(h, now.minute(), now.second()),
                            (E, m, E) => Arg::Time(now.hour(), m, now.second()),
                            (E, E, s) => Arg::Time(now.hour(), now.minute(), s),
                            (E, m, s) => Arg::Time(now.hour(), m, s),
                            (h, m, E) => Arg::Time(h, m, now.second()),
                            (h, m, s) => Arg::Time(h, m, s),
                        }
                    } else {
                        Arg::Title(arg.to_owned())
                    }
                }
            }
        };
        arg_toks.push(arg_tok);
    }

    arg_toks.into_iter().fold(Vec::new(), |mut acc, tok| {
        if let Some(last) = acc.last_mut() {
            match (last, &tok) {
                (Arg::Title(last), Arg::Title(tok)) => {
                    last.push(' ');
                    *last += &tok;
                }
                _ => acc.push(tok),
            }
        } else {
            acc.push(tok);
        }
        acc
    })
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let dir = directories::ProjectDirs::from("", "", "Remind").unwrap();
    let mut dir = dir.data_local_dir().to_owned();
    if dir.ends_with("data") {
        dir = dir.parent().unwrap().to_owned();
    }
    let file = dir.join("reminders.txt");
    let history_dir = dir.join("history");
    std::fs::create_dir_all(&dir).unwrap_or_default();
    std::fs::create_dir(&history_dir).unwrap_or_default();

    let mut reminder_file = ReminderFile::new(&file, &history_dir);
    if args.len() <= 1 {
        let file = file.to_str().unwrap_or_default();
        println!("reminders at: {file}");
        reminder_file.wait_next();
        return;
    }

    reminder_file.load();

    // Tokenize arguments
    let tokens = tokenize(&args);
    let mut title = String::new();
    let mut weekdays: u8 = 0;
    let mut repeats: u32 = 1;
    let now = Local::now();
    let mut end_time = now;
    let mut interval = Interval::default();
    let mut default_interval = Interval::default();
    for (i, tok) in tokens.iter().enumerate() {
        let mut add_time_unit = |unit: TimeUnit| {
            match unit {
                TimeUnit::Second(sec) => (
                    interval.secs += sec,
                    end_time += Duration::seconds(sec as i64),
                ),
                TimeUnit::Minute(min) => (
                    interval.mins += min,
                    end_time += Duration::minutes(min as i64),
                ),
                TimeUnit::Hour(hour) => (
                    interval.hours += hour,
                    end_time += Duration::hours(hour as i64),
                ),
                TimeUnit::Day(day) => {
                    (interval.days += day, end_time += Duration::days(day as i64))
                }
                TimeUnit::Week(week) => (
                    interval.days += 7 * week,
                    end_time += Duration::days(7 * week as i64),
                ),
                TimeUnit::Month(month) => (
                    interval.months += month,
                    end_time = end_time.checked_add_months(Months::new(month)).unwrap(),
                ),
                TimeUnit::Year(year) => (
                    interval.years += year,
                    end_time = end_time.with_year(end_time.year() + year as i32).unwrap(),
                ),
            };
        };
        let prev_tok = if i > 0 {
            tokens[i - 1].clone()
        } else {
            Arg::Help
        };
        let next_tok = if i < tokens.len() - 1 {
            tokens[i + 1].clone()
        } else {
            Arg::Help
        };
        match (prev_tok, tok.clone(), next_tok) {
            (_, Arg::List, _) => {
                reminder_file.list();
                return;
            }
            (_, Arg::Help, _) => {
                print_help();
                return;
            }
            (_, Arg::Undo, _) => {
                reminder_file.undo();
                return;
            }
            (_, Arg::Clear, _) => {
                reminder_file.save_history();
                reminder_file.clear();
                return;
            }
            (_, Arg::Remove, Arg::Title(titl)) | (Arg::Title(titl), Arg::Remove, _) => {
                reminder_file.save_history();
                reminder_file.remove(&titl);
                return;
            }
            // skip 3 "reminder" | skip "reminder" 3 | "reminder" skip 3
            // skip "reminder" | "reminder" skip | skip3 "reminder" | "reminder" skip3
            (Arg::Skip(0), Arg::Number(skips), Arg::Title(title))
            | (Arg::Skip(0), Arg::Title(title), Arg::Number(skips))
            | (Arg::Title(title), Arg::Skip(0), Arg::Number(skips))
            | (_, Arg::Skip(skips), Arg::Title(title))
            | (_, Arg::Title(title), Arg::Skip(skips)) => {
                reminder_file.save_history();
                reminder_file.skip(&title, skips.max(1));
                return;
            }
            // skip | skip 3 | skip3
            (_, Arg::Skip(0), Arg::Number(skips)) | (_, Arg::Skip(skips), _) => {
                reminder_file.save_history();
                reminder_file.skip_next(skips.max(1));
                return;
            }
            (_, Arg::Title(titl), _) => title = titl,
            (_, Arg::Repeat(0), Arg::Number(reps)) => repeats = reps,
            (_, Arg::Repeat(reps), _) => repeats = reps,
            (_, Arg::Month(month), Arg::Number(day)) => {
                end_time = end_time.with_month0(month).unwrap();
                end_time = end_time.with_day(day).unwrap();
                default_interval.years = 1;
            }
            (_, Arg::TimeUnit(mut unit), Arg::Number(time))
            | (Arg::Number(time), Arg::TimeUnit(mut unit), _) => {
                *unit = if *unit == 0 { time } else { *unit }.max(1);
                add_time_unit(unit);
            }
            (_, Arg::TimeUnit(mut unit), _) => {
                *unit = (*unit).max(1);
                add_time_unit(unit);
            }
            (_, Arg::Number(year), _) if year as i32 >= now.year() && year < 2200 => {
                end_time = end_time.with_year(year as i32).unwrap();
                default_interval.years = u32::MAX;
            }
            (_, Arg::WeekDay(bits), _) => {
                weekdays |= bits;
                default_interval.days = 1;
            }
            (_, Arg::Time(h, m, s), _) => {
                end_time = end_time
                    .with_time(NaiveTime::from_hms_opt(h, m, s).unwrap())
                    .unwrap();
                default_interval.days = 1;
            }
            (a, Arg::Month(_), b) => panic!(
                "invalid month pattern ({a:?} Arg::Month {b:?}), try: remind july 4 \"my reminder\""
            ),
            (a, Arg::Remove, b) => {
                panic!(
                    "invalid remove pattern ({a:?} Arg::Remove {b:?}), try: remove \"my reminder\""
                )
            }
            (_, Arg::Number(_), _) => {}
        };
    }

    if interval.is_zero() {
        if default_interval.years == u32::MAX {
            repeats = 1;
        } else if default_interval.years == 1 {
            interval.years = 1;
        } else if default_interval.months == 1 {
            interval.months = 1;
        } else {
            interval.days = 1;
        }
    }

    let mut reminder = Reminder {
        title,
        interval,
        end_time,
        repeats,
        skips: 0,
        weekdays,
    };
    reminder.update();
    reminder_file.save_history();
    reminder_file.append(&reminder);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_num_str_num() {
        assert_eq!(num_str_num("32week4"), (32, "week".to_string(), 4));
        assert_eq!(num_str_num("10days"), (10, "days".to_string(), 0));
        assert_eq!(num_str_num("months5"), (0, "months".to_string(), 5));
        assert_eq!(num_str_num("year"), (0, "year".to_string(), 0));
    }

    fn to_args(str: &[&str]) -> Vec<String> {
        str.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_tokenize() {
        let args = to_args(&["remind", "3w", "write homework"]);
        let tokens = tokenize(&args);
        assert_eq!(
            tokens,
            vec![
                Arg::TimeUnit(TimeUnit::Week(3)),
                Arg::Title("write homework".to_string())
            ]
        );

        let args = to_args(&["remind", "1m", "egg ready", "rep4", "skip", "3"]);
        let tokens = tokenize(&args);
        assert_eq!(
            tokens,
            [
                Arg::TimeUnit(TimeUnit::Minute(1)),
                Arg::Title("egg ready".to_string()),
                Arg::Repeat(4),
                Arg::Skip(0),
                Arg::Number(3)
            ]
        );

        let args = to_args(&["remind", "july", "4", "pay", "12:30"]);
        let tokens = tokenize(&args);
        let now = Local::now();
        assert_eq!(
            tokens,
            vec![
                Arg::Month(6),
                Arg::Number(4),
                Arg::Title("pay".to_string()),
                Arg::Time(12, 30, now.second())
            ]
        );
    }

    #[test]
    fn test_print_help() {
        print_help();
    }

    #[test]
    fn test_reminder_file() {
        let mut reminder_file = ReminderFile::new("test_reminders.txt", "test_history");
        reminder_file.append(&Reminder {
            title: "Test Reminder".to_string(),
            interval: Interval::default(),
            end_time: Local::now(),
            repeats: 0,
            skips: 0,
            weekdays: 0,
        });
        reminder_file.list();
        reminder_file.remove("test rem");
        std::fs::remove_file("test_reminders.txt").unwrap();
    }
}
