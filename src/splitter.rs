use influx::{
    extract_timestamp, get_range, influx_client, json_val_to_influx_val, save_points, Error,
    SeriesResult,
};
use influx_db_client::Point;
use rayon::prelude::*;
use settings::Settings;
use std::process::exit;
use time::Duration;
use trade::pair_names;
use utils::time::intervals;

//#[derive(Fail, Debug)]
//pub enum Error {
//}

pub fn split(settings: &Settings) -> () {
    let client = influx_client();

    // Hey look, par_iter() !!
    pair_names().par_iter()
//        .take(1)
        .for_each(|pair_name| {
            println!("start {}", pair_name);

            let measurement = format!("trades_binance_{pair_name}_raw", pair_name = pair_name);

            for (_i, (start, end)) in intervals(settings.start, settings.end, Duration::hours(1))
                .enumerate()
//            .take(1)
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

                    let _count = series.values.iter().count();

//                    println!("{} - [{} - {}] ({})", i, start, end, count);

                    let points = to_points(series, &measurement).unwrap_or_else(|e| {
                        println!("Error: {}", e);
                        exit(-1)
                    });

//                println!("{:#?}", points);

                    // TODO: handle errors
                    save_points(&client, "glukoz-rp", points).unwrap();
//                        {
//                        Err(e) => {
//
//                        }
//                        _ => continue
//                    }
                }
            println!("end {}", pair_name);
        });
}

pub fn to_points(series: SeriesResult, measurement: &str) -> Result<Vec<Point>, Error> {
    series
        .values
        .iter()
        .map(|record| {
            let mut point = Point::new(measurement);

            let ts = extract_timestamp(&record)?;
            point.add_timestamp(ts);

            for (val, column_name) in record
                .into_iter()
                .skip(1)
                .zip(series.columns.iter().skip(1))
            {
                let influx_val = json_val_to_influx_val(val)?;
                point.add_field(column_name, influx_val);
            }

            Ok(point)
        })
        .collect()
}
