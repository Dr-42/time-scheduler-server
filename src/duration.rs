use serde::{Deserialize, Serialize};
use std::ops;

#[derive(Debug, Serialize, Deserialize, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct Duration {
    pub seconds: u64,
    pub minutes: u64,
    pub hours: u64,
}

impl Duration {
    pub fn to_seconds(&self) -> u64 {
        self.seconds + self.minutes * 60 + self.hours * 3600
    }

    pub fn from_seconds(seconds: u64) -> Self {
        let hours = seconds / 3600;
        let minutes = (seconds - hours * 3600) / 60;
        let seconds = seconds - hours * 3600 - minutes * 60;
        Self {
            seconds,
            minutes,
            hours,
        }
    }
}

impl ops::Add for Duration {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            seconds: self.seconds + other.seconds,
            minutes: self.minutes + other.minutes,
            hours: self.hours + other.hours,
        }
    }
}

impl ops::Sub for Duration {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            seconds: self.seconds - other.seconds,
            minutes: self.minutes - other.minutes,
            hours: self.hours - other.hours,
        }
    }
}
