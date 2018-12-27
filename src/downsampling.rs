use chrono::NaiveDateTime;
use crate::settings::Interval;
use crate::{
    influx::{
        extract_float_value, from_json_values, get_range, save_points, to_point, Error, FieldValue,
    },
    lttb::{lttb_downsample, DataPoint},
    settings::{Config, Field},
    utils::{error::print_err_and_exit, time::UNIX_EPOCH},
};
use influx_db_client::Client;
use influx_db_client::Point;
use std::collections::HashMap;
use std::ops::Sub;
use std::time::Duration as StdDuration;
use string_template::Template;
use time::Duration;

pub fn downsample_period(
    config: &Config,
    client: &Client,
    query_template: &Template,
    id: &str,
    end: NaiveDateTime,
    interval_duration_secs: u64,
    measurement_name: &str,
) {
    // TODO: batch small periods queries into large ones
    let duration = Duration::from_std(StdDuration::from_secs(interval_duration_secs)).unwrap();
    let begin = end.sub(duration);

    // TODO: attempt to downsample from downsampled series instead of from 'raw'
    let query_str = build_query(&query_template, id, begin, end, 0, "raw");
    let series = match get_range(&client, &query_str) {
        Ok(series) => series,
        Err(err) => match err {
            Error::NoResult => return,
            e => print_err_and_exit(e),
        },
    };
    let vals = from_json_values(series.values, &config.downsampler.fields)
        .unwrap_or_else(|e| print_err_and_exit(e));
    //                let _count = vals.iter().count();
    //                println!("{} - [{} - {}] ({})", i, start, end, _count);
    let subset = lttb_downsample(
        &vals,
        60,
        config.downsampler.x_field_index,
        config.downsampler.y_field_index,
    );
    let points = to_influx_points(measurement_name, &vals, &subset, &config.downsampler.fields);
    //                println!("{:#?}", &points);
    // TODO: handle errors
    save_points(&client, &config.influxdb.retention_policy, points).unwrap();
}

pub fn to_influx_points(
    measurement_name: &str,
    raw: &[Vec<FieldValue>],
    downsampled: &Option<Vec<&Vec<FieldValue>>>,
    fields: &[Field],
) -> Vec<Point> {
    match downsampled {
        Some(downsampled) => downsampled
            .iter()
            .map(|v| to_point(v, measurement_name, fields))
            .collect(),
        _ => raw
            .iter()
            .map(|v| to_point(v, measurement_name, fields))
            .collect(),
    }
}

// pass `limit: 0` to disable limit
pub fn build_query(
    query_template: &Template,
    id: &str,
    start: NaiveDateTime,
    end: NaiveDateTime,
    limit: i64,
    time_interval: &str,
) -> String {
    let start_str = start.timestamp_nanos().to_string();
    let end_str = end.timestamp_nanos().to_string();
    let limit_str = limit.to_string();

    let mut map = HashMap::new();
    map.insert("id", id);
    map.insert("start", &start_str);
    map.insert("end", &end_str);
    map.insert("limit", &limit_str);
    map.insert("time_interval", time_interval);

    query_template.render(&map)
}

impl DataPoint for Vec<FieldValue> {
    fn get_x(&self, index: usize) -> f64 {
        let field_value = self.get(index).unwrap();
        extract_float_value(field_value)
    }

    fn get_y(&self, index: usize) -> f64 {
        let field_value = self.get(index).unwrap();
        extract_float_value(field_value)
    }
}

pub fn is_downsampling_interval(start: &NaiveDateTime, interval_period: &Interval) -> bool {
    let secs = start.signed_duration_since(*UNIX_EPOCH).num_seconds();
    secs % (interval_period.duration_secs as i64) == 0
}
