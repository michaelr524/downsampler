use chrono::NaiveDateTime;
use crate::cmdargs::TimePeriod;
use crate::influx::from_json_values;
use crate::influx::to_point;
use crate::influx::FieldValue;
use crate::influx::{get_range, influx_client, save_points, Error};
use crate::settings::Config;
use crate::settings::Field;
use crate::utils::error::print_err_and_exit;
use crate::utils::time::intervals;
use influx_db_client::Point;
use rayon::prelude::*;
use std::collections::HashMap;
use string_template::Template;
use time::Duration;

//#[derive(Fail, Debug)]
//pub enum Error {
//}

pub fn split(args: &TimePeriod, config: &Config) -> () {
    let client = influx_client(
        &config.influxdb.url,
        &config.influxdb.db,
        &config.influxdb.username,
        &config.influxdb.pass,
    );

    let measurement_template = Template::new(&config.splitter.measurement_template);
    let query_template = Template::new(&config.splitter.query_template);

    // Hey look, par_iter() !!
    config
        .vars
        .ids
        .par_iter()
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
                            e => print_err_and_exit(e)
                        },
                    };

                    let _count = series.values.iter().count();

//                    println!("{} - [{} - {}] ({})", i, start, end, count);

                    let vals = from_json_values(&series.values, &config.splitter.fields)
                        .unwrap_or_else(|e| print_err_and_exit(e));

                    let points = to_points(&vals, &measurement_name, &config.splitter.fields);

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

pub fn to_points(
    vals: &Vec<Vec<FieldValue>>,
    measurement: &str,
    fields: &Vec<Field>,
) -> Vec<Point> {
    vals.iter()
        .map(|record| to_point(record, measurement, fields))
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
