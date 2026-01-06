use chrono::{Datelike, NaiveDate};
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq)]
pub struct Birthday {
    day: u8,
    month: u8,
}

impl Birthday {
    pub fn matches(&self, date: &NaiveDate) -> bool {
        date.day() == self.day as u32 && date.month() == self.month as u32
    }
}
