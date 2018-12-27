use chrono::{Datelike, NaiveDateTime, TimeZone, Timelike, Utc};
use lazy_static::lazy_static;
use time::Duration;

lazy_static! {
    pub static ref UNIX_EPOCH: NaiveDateTime = Utc.timestamp(0, 0).naive_utc();
}

pub fn truncate_seconds(dt: NaiveDateTime) -> NaiveDateTime {
    Utc.ymd(dt.year(), dt.month(), dt.day())
        .and_hms(dt.hour(), dt.minute(), 0u32)
        .naive_utc()
}

pub fn parse_timestamp_sec(timestamp_str: &str) -> NaiveDateTime {
    let timestamp = timestamp_str.parse::<i64>().unwrap();
    NaiveDateTime::from_timestamp(timestamp, 0)
}

pub struct IntervalIterator {
    pub end: NaiveDateTime,
    pub cur: NaiveDateTime,
    pub prev: NaiveDateTime,
    pub step: Duration,
}

impl Iterator for IntervalIterator {
    type Item = (NaiveDateTime, NaiveDateTime);

    fn next(&mut self) -> Option<Self::Item> {
        self.prev = self.cur;
        self.cur = self.cur + self.step;

        if self.cur <= self.end {
            Some((self.prev, self.cur))
        } else {
            None
        }
    }
}

pub fn intervals(start: NaiveDateTime, end: NaiveDateTime, step: Duration) -> IntervalIterator {
    IntervalIterator {
        end,
        cur: start,
        prev: start,
        step,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test() {
        let now = Utc::now().naive_utc();
        let time = truncate_seconds(now);
        assert_eq!(time.timestamp_subsec_nanos(), 0u32);
    }
}

//1538863449575227000
//1538863449575
//1538863449
