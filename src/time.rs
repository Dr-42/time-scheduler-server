use crate::{duration::Duration, Result};
use serde::{Deserialize, Serialize};
use std::ops::Add;

#[derive(Debug, Serialize, Deserialize, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct Time {
    pub year: u32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl Time {
    pub fn new(year: u32, month: u8, day: u8, hour: u8, minute: u8, second: u8) -> Result<Self> {
        if !matches!(month, 1..=12) {
            return Err("Invalid month".into());
        }
        let days_month = if Time::is_leap_year(year) {
            [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 31, 31]
        } else {
            [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 31, 31]
        };
        if day < 1 || day > days_month[month as usize - 1] {
            return Err("Invalid day".into());
        }
        if hour > 23 {
            return Err("Invalid hour".into());
        }
        if minute > 59 {
            return Err("Invalid minute".into());
        }
        if second > 59 {
            return Err("Invalid second".into());
        }
        Ok(Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
        })
    }

    fn is_leap_year(year: u32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
    }

    pub fn to_iso(&self) -> u64 {
        let unix_start = Time::new(1970, 1, 1, 0, 0, 0).unwrap();
        let days_month = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        const SECONDS_PER_DAY: u64 = 24 * 60 * 60;
        let mut days = 0;
        for year in unix_start.year..self.year {
            days += if Time::is_leap_year(year) { 366 } else { 365 };
        }
        for month in 1..self.month {
            days += days_month[month as usize - 1] as u64;
        }
        if self.month > 2 && Time::is_leap_year(self.year) {
            days += 1;
        }
        days += self.day as u64 - 1;
        let mut seconds = days * SECONDS_PER_DAY;
        seconds += self.hour as u64 * 60 * 60;
        seconds += self.minute as u64 * 60;
        seconds += self.second as u64;
        seconds
    }

    pub fn from_iso(seconds: u64) -> Self {
        let unix_start = Time::new(1970, 1, 1, 0, 0, 0).unwrap();
        let mut seconds = seconds;
        let mut year = unix_start.year;
        let mut month = 1;
        let mut day = 1;
        let mut hour = 0;
        let mut minute = 0;
        const SECONDS_PER_DAY: u64 = 24 * 60 * 60;
        while seconds >= SECONDS_PER_DAY {
            let days = if Time::is_leap_year(year) { 366 } else { 365 };
            if seconds >= days * SECONDS_PER_DAY {
                seconds -= days * SECONDS_PER_DAY;
                year += 1;
            } else {
                break;
            }
        }
        let days_month = if Time::is_leap_year(year) {
            [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 31, 31]
        } else {
            [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 31, 31]
        };
        while seconds >= SECONDS_PER_DAY {
            if seconds >= days_month[month as usize - 1] as u64 * SECONDS_PER_DAY {
                seconds -= days_month[month as usize - 1] as u64 * SECONDS_PER_DAY;
                month += 1;
            } else {
                break;
            }
        }
        while seconds >= SECONDS_PER_DAY {
            if seconds >= SECONDS_PER_DAY {
                seconds -= SECONDS_PER_DAY;
                day += 1;
            } else {
                break;
            }
        }
        while seconds >= 60 * 60 {
            if seconds >= 60 * 60 {
                seconds -= 60 * 60;
                hour += 1;
            } else {
                break;
            }
        }
        while seconds >= 60 {
            if seconds >= 60 {
                seconds -= 60;
                minute += 1;
            } else {
                break;
            }
        }
        let second = seconds as u8;
        Self {
            year,
            month: month as u8,
            day: day as u8,
            hour: hour as u8,
            minute: minute as u8,
            second,
        }
    }

    pub fn time_span(&self, other: &Time) -> Duration {
        let self_seconds = self.to_iso();
        let other_seconds = other.to_iso();
        let seconds = if self_seconds > other_seconds {
            self_seconds - other_seconds
        } else {
            other_seconds - self_seconds
        };
        Duration::from_seconds(seconds)
    }

    pub fn get_previous_day(&self) -> Self {
        let t2 = self.to_iso() - 24 * 60 * 60;
        Time::from_iso(t2)
    }

    pub fn filename(&self) -> String {
        format!(
            "timeblocks/{:04}-{:02}-{:02}.json",
            self.year, self.month, self.day
        )
    }

    pub fn before(&self, other: &Time) -> bool {
        self.to_iso() < other.to_iso()
    }
}

impl Add<Duration> for Time {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self::Output {
        let seconds = self.to_iso() + rhs.to_seconds();
        Time::from_iso(seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leaps() {
        assert_eq!(Time::is_leap_year(2020), true);
        assert_eq!(Time::is_leap_year(2021), false);
        assert_eq!(Time::is_leap_year(2022), false);
        assert_eq!(Time::is_leap_year(2023), false);
        assert_eq!(Time::is_leap_year(2024), true);
        assert_eq!(Time::is_leap_year(1804), true);
        assert_eq!(Time::is_leap_year(1800), false);
        assert_eq!(Time::is_leap_year(1932), true);
    }

    #[test]
    fn test_iso() {
        let time = Time::new(2023, 11, 27, 3, 18, 52).to_iso();
        let iso = 1701055132;
        assert_eq!(iso, time);
        let time = Time::from_iso(iso);
        assert_eq!(time, Time::new(2023, 11, 27, 3, 18, 52));
    }
}
