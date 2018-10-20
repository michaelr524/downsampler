use chrono::Datelike;
use chrono::NaiveDateTime;
use chrono::TimeZone;
use chrono::Timelike;
use chrono::Utc;
use time::Duration;

pub fn truncate_seconds(dt: NaiveDateTime) -> NaiveDateTime {
    Utc.ymd(dt.year(), dt.month(), dt.day())
        .and_hms(dt.hour(), dt.minute(), 0u32)
        .naive_utc()
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
