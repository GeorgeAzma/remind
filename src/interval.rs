#[derive(Default, PartialEq, Eq)]
pub struct Interval {
    pub secs: u32,
    pub mins: u32,
    pub hours: u32,
    pub days: u32,
    pub months: u32,
    pub years: u32,
}

impl Interval {
    pub fn deserialize(str: &str) -> Self {
        Self {
            secs: str[15..17].parse().unwrap(),
            mins: str[12..14].parse().unwrap(),
            hours: str[9..11].parse().unwrap(),
            days: str[6..8].parse().unwrap(),
            months: str[3..5].parse().unwrap(),
            years: str[0..2].parse().unwrap(),
        }
    }

    pub fn serialize(&self) -> String {
        format!(
            "{:02}-{:02}-{:02} {:02}:{:02}:{:02}",
            self.years, self.months, self.days, self.hours, self.mins, self.secs
        )
    }

    pub fn is_zero(&self) -> bool {
        self.secs == 0
            && self.mins == 0
            && self.hours == 0
            && self.days == 0
            && self.months == 0
            && self.years == 0
    }
}
