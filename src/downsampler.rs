use influx::Error;
use influx::{get_range, influx_client, save_points};
use influx_db_client::Point;
use lttb::lttb_downsample;
use rayon::prelude::*;
use serde_json::Value;
use settings::Settings;
use std::process::exit;
use time::Duration;
use trade::{pair_names, Trade};
use utils::time::intervals;

pub fn downsample(settings: &Settings) -> () {
    let client = influx_client();

    // Hey look, par_iter() !!
    pair_names().par_iter().take(5).for_each(|pair_name| {
        println!("start {}", pair_name);

        for (i, (start, end)) in intervals(settings.start, settings.end, Duration::seconds(60))
            .enumerate()
            .take(1)
        {
            let series = match get_range(&client, pair_name, start, end) {
                Ok(series) => series,
                Err(err) => match err {
                    Error::NoResult => continue,
                    e => {
                        println!("Error: {}", e);
                        exit(-1)
                    }
                },
            };

            let count = series.values.iter().count();
            println!("{} - [{} - {}] ({})", i, start, end, count);

            let raw = to_trades(&series.values);
            let downsampled = lttb_downsample(&raw, 60);
            let points = to_points(&raw, &downsampled);

            //                println!("{:#?}", &points);

            // TODO: handle errors
            save_points(&client, "glukoz-rp", points).unwrap();
        }

        println!("end {}", pair_name);
    });
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

//pub struct TimeSeries<'a> {
//    interval_sec: i64,
//    source_series: &'a str,
//    dest_series: &'a str,
//}
//
//pub fn series<'a>() -> [TimeSeries<'static>; 2] {
//    [
//        TimeSeries {
//            interval_sec: 10,
//            source_series: "",
//            dest_series: "binance_btcusdt_trades_10s",
//        },
//        TimeSeries {
//            interval_sec: 10,
//            source_series: "",
//            dest_series: "binance_btcusdt_trades_seconds",
//        },
//    ]
//}
