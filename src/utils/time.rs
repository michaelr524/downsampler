use chrono::Datelike;
use chrono::NaiveDateTime;
use chrono::TimeZone;
use chrono::Timelike;
use chrono::Utc;

pub fn truncate_seconds(dt: NaiveDateTime) -> NaiveDateTime {
    Utc.ymd(dt.year(), dt.month(), dt.day())
        .and_hms(dt.hour(), dt.minute(), 0u32)
        .naive_utc()
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
