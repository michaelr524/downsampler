use chrono::NaiveDateTime;
use cmdargs::CmdArgs;
use influx::FieldValue;
use influx::{
    extract_float_value, extract_int_value, from_json_values, get_range, influx_client, field_val_to_influx_val,
    save_points, Error,
};
use influx_db_client::Point;
use lttb::{lttb_downsample, DataPoint};
use rayon::prelude::*;
use settings::Config;
use settings::Field;
use std::collections::HashMap;
use string_template::Template;
use time::Duration;
use utils::error::print_err_and_exit;
use utils::time::intervals;

pub fn downsample(args: &CmdArgs, config: &Config) -> () {
    let client = influx_client(
        &config.influxdb.url,
        &config.influxdb.db,
        &config.influxdb.username,
        &config.influxdb.pass,
    );

    let measurement_template = Template::new(&config.downsampler.measurement_template);
    let query_template = Template::new(&config.downsampler.query_template);

    //    Hey look, par_iter() !!
    config.vars.ids.par_iter().take(16).for_each(|id| {
        println!("start {}", id);

        let measurement_name = make_measurement_name(&measurement_template, id, "seconds");

        for (start, end)
//            (_i, (start, end))
            in
            intervals(args.start, args.end, Duration::seconds(60))
//            .enumerate()
//            .take(1)
            {
                let query_str = build_query(&query_template, id, start, end, 0, "raw");
                let series = match get_range(&client, &query_str) {
                    Ok(series) => series,
                    Err(err) => match err {
                        Error::NoResult => continue,
                        e => print_err_and_exit(e)
                    },
                };

                let vals = from_json_values(series.values, &config.downsampler.fields)
                    .unwrap_or_else(|e| print_err_and_exit(e));

//                let _count = vals.iter().count();
//                println!("{} - [{} - {}] ({})", i, start, end, _count);

                let subset = lttb_downsample(&vals,
                                             60,
                                             config.downsampler.x_field_index,
                                             config.downsampler.y_field_index);
                let points = to_influx_points(&measurement_name,
                                              &vals,
                                              &subset,
                                              &config.downsampler.fields);

//                println!("{:#?}", &points);

                // TODO: handle errors
                save_points(&client, &config.influxdb.retention_policy, points).unwrap();
            }

        println!("end {}", id);
    });
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

pub fn make_measurement_name(template: &Template, id: &str, time_interval: &str) -> String {
    let mut map = HashMap::new();
    map.insert("id", id);
    map.insert("time_interval", time_interval);
    template.render(&map)
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

pub fn to_point(v: &Vec<FieldValue>, measurement: &str, fields: &Vec<Field>) -> Point {
    let mut point = Point::new(measurement);

    let timestamp = extract_int_value(&v[0]);
    point.add_timestamp(timestamp);

    for (val, field) in v.iter().skip(1).zip(fields.iter()) {
        let influx_val = field_val_to_influx_val(val);
        point.add_field(&field.name, influx_val);
    }

    point
}
