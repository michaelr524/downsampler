use chrono::{NaiveDateTime, TimeZone, Utc};
use crate::{
    cmdargs::CmdArgs,
    influx::{
        extract_float_value, from_json_values, get_range, influx_client, save_points, to_point,
        Error, FieldValue,
    },
    lttb::{lttb_downsample, DataPoint},
    settings::{Config, Field},
    utils::{error::print_err_and_exit, time::intervals},
};
use influx_db_client::Client;
use influx_db_client::Point;
use lazy_static::lazy_static;
use rayon::prelude::*;
use std::collections::HashMap;
use string_template::Template;
use time::Duration;
use std::time::Duration as StdDuration;
use std::ops::{Sub};

lazy_static! {
    static ref UNIX_EPOCH: NaiveDateTime = Utc.timestamp(0, 0).naive_utc();
}

pub fn pre_render_names(config: &Config, template: Template) -> HashMap<(u64, &str), String> {
    let mut map: HashMap<(u64, &str), String> =
        HashMap::with_capacity(config.vars.ids.len() * config.downsampler.intervals.len());

    for id in &config.vars.ids {
        for interval in &config.downsampler.intervals {
            let mut m = HashMap::new();
            m.insert("id", id.as_str());
            m.insert("time_interval", interval.name.as_str());
            let name = template.render(&m);
            map.insert((interval.duration_secs, id.as_str()), name);
        }
    }

    map
}

pub fn downsample(args: &CmdArgs, config: &Config) -> () {
    let client = influx_client(
        &config.influxdb.url,
        &config.influxdb.db,
        &config.influxdb.username,
        &config.influxdb.pass,
    );

    let measurement_template = Template::new(&config.downsampler.measurement_template);
    let query_template = Template::new(&config.downsampler.query_template);
    let measurements = pre_render_names(&config, measurement_template);

    //    Hey look, par_iter() !!
    config.vars.ids.par_iter().for_each(|id| {
        println!("start {}", id);

        for (start, end) in intervals(args.start, args.end, Duration::seconds(1)) {
            for interval_period in config.downsampler.intervals.iter() {
                if start.signed_duration_since(*UNIX_EPOCH).num_seconds()
                    % (interval_period.duration_secs as i64)
                    == 0
                {
                    let measurement_name = measurements
                        .get(&(interval_period.duration_secs, id))
                        .unwrap();

                    downsample_period(
                        config,
                        &client,
                        &query_template,
                        id,
                        start,
                        interval_period.duration_secs,
                        measurement_name,
                    );
                }
            }
        }

        println!("end {}", id);
    });
}

pub fn downsample_period(
    config: &Config,
    client: &Client,
    query_template: &Template,
    id: &str,
    end: NaiveDateTime,
    interval_duration_secs: u64,
    measurement_name: &str,
) {
    let duration = Duration::from_std(StdDuration::from_secs(interval_duration_secs)).unwrap();
    let begin = end.sub(duration);

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
    raw: &Vec<Vec<FieldValue>>,
    downsampled: &Option<Vec<&Vec<FieldValue>>>,
    fields: &Vec<Field>,
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
