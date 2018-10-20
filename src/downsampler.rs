use influx::{build_query, get_values, influx_client, run_query, save_points};
use influx_db_client::Point;
use lttb::lttb_downsample;
use serde_json::Value;
use settings::parse_args;
use std::process::exit;
use time::Duration;
use trade::Trade;
use utils::time::{IntervalIterator};
use settings::{Settings, print_settings_info};

pub fn downsample(settings: &Settings) -> () {
    let iter = IntervalIterator {
        end: settings.end,
        cur: settings.start,
        prev: settings.start,
        step: Duration::seconds(60),
    };

    let client = influx_client();

    for (i, (start, end)) in iter.enumerate().take(3) {
        let query = build_query(start.timestamp_nanos(), end.timestamp_nanos(), 0);
        let res = run_query(&client, &query);
        let vals = get_values(&res);

        let count = vals.iter().count();

        println!("{} - [{} - {}] ({})", i, start, end, count);

        let raw = to_trades(vals);
        let downsampled = lttb_downsample(&raw, 60);
        let points = to_points(&raw, &downsampled);

        //        println!("{:#?}", points);

        // TODO: handle errors
        save_points(&client, "glukoz-rentention-policy", points).unwrap();
    }
}

pub fn to_trades(vals: &Vec<Vec<Value>>) -> Vec<Trade> {
    vals.iter()
        .map(|val| Trade {
            price: val[1].as_f64().unwrap(),
            timestamp: val[0].as_i64().unwrap(),
            amount: val[2].as_f64().unwrap(),
        })
        .collect()
}

pub fn to_points(raw: &Vec<Trade>, downsampled: &Option<Vec<&Trade>>) -> Vec<Point> {
    let points: Vec<Point> = if let Some(downsampled) = downsampled {
        downsampled
            .iter()
            .map(|trade| trade.to_point("binance_btcusdt_trades_seconds"))
            .collect()
    } else {
        raw.iter()
            .map(|trade| trade.to_point("binance_btcusdt_trades_seconds"))
            .collect()
    };

    points
}

pub struct TimeSeries<'a> {
    interval_sec: i64,
    source_series: &'a str,
    dest_series: &'a str,
}

pub fn series<'a>() -> [TimeSeries<'static>; 2] {
    [
        TimeSeries {
            interval_sec: 10,
            source_series: "",
            dest_series: "binance_btcusdt_trades_10s",
        },
        TimeSeries {
            interval_sec: 10,
            source_series: "",
            dest_series: "binance_btcusdt_trades_seconds",
        },
    ]
}