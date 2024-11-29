use std::ops::{Deref, DerefMut};

#[derive(Clone, PartialEq, Eq)]
pub enum TimeUnit {
    Second(u32),
    Minute(u32),
    Hour(u32),
    Day(u32),
    Week(u32),
    Month(u32),
    Year(u32),
}

impl Deref for TimeUnit {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Second(val)
            | Self::Minute(val)
            | Self::Hour(val)
            | Self::Day(val)
            | Self::Week(val)
            | Self::Month(val)
            | Self::Year(val) => val,
        }
    }
}

impl DerefMut for TimeUnit {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Second(val)
            | Self::Minute(val)
            | Self::Hour(val)
            | Self::Day(val)
            | Self::Week(val)
            | Self::Month(val)
            | Self::Year(val) => val,
        }
    }
}

impl std::fmt::Debug for TimeUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let unit = match self {
            Self::Second(_) => "second",
            Self::Minute(_) => "minute",
            Self::Hour(_) => "hour",
            Self::Day(_) => "day",
            Self::Week(_) => "week",
            Self::Month(_) => "month",
            Self::Year(_) => "year",
        };
        let time = *self.deref();
        f.write_fmt(format_args!("{time} {unit}"))
    }
}
