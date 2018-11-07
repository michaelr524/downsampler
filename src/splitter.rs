use chrono::NaiveDateTime;
use crate::cmdargs::CmdArgs;
use crate::influx::{
    extract_timestamp, get_range, influx_client, field_val_to_influx_val, save_points, Error,
    SeriesResult,
};
use influx_db_client::Point;
use rayon::prelude::*;
use crate::settings::Config;
use std::collections::HashMap;
use std::process::exit;
use string_template::Template;
use time::Duration;
use crate::utils::time::intervals;

//#[derive(Fail, Debug)]
//pub enum Error {
//}

pub fn split(args: &CmdArgs, config: &Config) -> () {
    let client = influx_client(
        &config.influxdb.url,
        &config.influxdb.db,
        &config.influxdb.username,
        &config.influxdb.pass,
    );

    let measurement_template = Template::new(&config.splitter.measurement_template);
    let query_template = Template::new(&config.splitter.query_template);

    // Hey look, par_iter() !!
    config.vars.ids.par_iter()
//        .take(1)
        .for_each(|id| {
            println!("start {}", id);

            let measurement_name = make_measurement_name(&measurement_template, id);

            for (start, end)
//                (_i, (start, end))
                in
                intervals(args.start, args.end, Duration::hours(1))
//                .enumerate()
//                .take(1)
                {
                    let query_str = build_query(&query_template, id, start, end, 0);
                    let series = match get_range(&client, &query_str) {
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

                    let points = to_points(series, &measurement_name).unwrap_or_else(|e| {
                        println!("Error: {}", e);
                        exit(-1)
                    });

//                println!("{:#?}", points);

                    // TODO: handle errors
                    save_points(&client, &config.influxdb.retention_policy, points).unwrap();
//                        {
//                        Err(e) => {
//
//                        }
//                        _ => continue
//                    }
                }
            println!("end {}", id);
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
                // TODO: fix this to use the same as downsampler
                //                let influx_val = json_val_to_influx_val(val)?;
                //                point.add_field(column_name, influx_val);
            }

            Ok(point)
        })
        .collect()
}

pub fn make_measurement_name(template: &Template, id: &str) -> String {
    let mut map = HashMap::new();
    map.insert("id", id);
    template.render(&map)
}

// pass `limit: 0` to disable limit
pub fn build_query(
    query_template: &Template,
    id: &str,
    start: NaiveDateTime,
    end: NaiveDateTime,
    limit: i64,
) -> String {
    let start_str = start.timestamp_nanos().to_string();
    let end_str = end.timestamp_nanos().to_string();
    let limit_str = limit.to_string();

    let mut map = HashMap::new();
    map.insert("id", id);
    map.insert("start", &start_str);
    map.insert("end", &end_str);
    map.insert("limit", &limit_str);

    query_template.render(&map)
}
