extern crate chrono;
extern crate clap;
extern crate env_logger;
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate humantime;
extern crate time;
//#[macro_use]
extern crate influx_db_client;
extern crate kairos;
extern crate serde_json;

mod query;
mod settings;
mod utils;

use chrono::NaiveDateTime;
use influx_db_client::Client;
use query::{build_query, run_query};
use settings::parse_args;
use std::process::exit;
use time::Duration;
use utils::time::truncate_seconds;

fn main() {
    let mut settings = parse_args().unwrap_or_else(|e| {
        println!("Error: {}", e);
        exit(-1)
    });

    println!("Period {:?} - {:?}", settings.start, settings.end);

    settings.start = truncate_seconds(settings.start);
    settings.end = truncate_seconds(settings.end);

    println!("Period {:?} - {:?}", settings.start, settings.end);

    let start = settings.start.timestamp_nanos();
    let end = settings.end.timestamp_nanos();

    println!("Period in nanos {:?} - {:?}", start, end);

    let iter = IntervalIterator {
        end: settings.end,
        cur: settings.start,
        prev: settings.start,
        step: Duration::seconds(60),
    };

    let client = Client::default().set_authentication("root", "root");

    for (i, (start, end)) in iter.enumerate().take(1) {
        let query = build_query(start.timestamp_nanos(), end.timestamp_nanos(), 0);
        let res = run_query(&client, &query);
        let vals = &res
            .as_ref()
            .unwrap()
            .as_ref()
            .unwrap()
            .first()
            .unwrap()
            .series
            .as_ref()
            .unwrap()
            .first()
            .as_ref()
            .unwrap()
            .values;

        let count = vals.iter().count();

        println!("{} - [{} - {}] ({})", i, start, end, count);

        for val in vals {
            let timestamp = &val[0].as_i64().unwrap();
            let price = &val[1].as_f64().unwrap();
            let amount = &val[2].as_f64().unwrap();
            println!("{} {} {}", timestamp, price, amount);
        }
    }
}

struct IntervalIterator {
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
