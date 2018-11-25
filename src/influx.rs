use crate::{settings::Field, settings::FieldDataType};
use failure_derive::Fail;
use influx_db_client::{error, Client, Node, Point, Points, Precision, Value as InfluxValue};
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub enum FieldValue {
    Float(f64),
    Integer(i64),
    Boolean(bool),
    String(String),
}

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "Failed to access InfluxDB: Error: {:?}", _0)]
    InfluxDbAccessError(error::Error),
    #[fail(display = "InfluxDB query didn't return a result.")]
    NoResult,
    #[fail(display = "InfluxDB returned a field with null value.")]
    UnexpectedDataType(String, Value),
}

pub struct SeriesResult {
    pub values: Vec<Vec<Value>>,
    pub columns: Vec<String>,
}

pub fn influx_client(url: &str, db_name: &str, username: &str, pass: &str) -> Client {
    Client::new(url, db_name).set_authentication(username, pass)
}

pub fn run_query(client: &Client, query: &str) -> Result<Option<Vec<Node>>, error::Error> {
    //    println!("{}", query);

    client.query(query, Some(Precision::Nanoseconds))
}

pub fn first_series_from_result(
    result: Result<Option<Vec<Node>>, error::Error>,
) -> Result<SeriesResult, Error> {
    let first_series = result
        .map_err(|e| Error::InfluxDbAccessError(e))?
        .ok_or_else(|| Error::NoResult)?
        .into_iter()
        .next()
        .ok_or_else(|| Error::NoResult)?
        .series
        .ok_or_else(|| Error::NoResult)?
        .into_iter()
        .next()
        .ok_or_else(|| Error::NoResult)?;

    Ok(SeriesResult {
        values: first_series.values,
        columns: first_series.columns,
    })
}

pub fn get_range(client: &Client, query_str: &str) -> Result<SeriesResult, Error> {
    let res = run_query(&client, query_str);
    first_series_from_result(res)
}

pub fn from_json_values(
    vals: Vec<Vec<Value>>,
    fields: &Vec<Field>,
) -> Result<Vec<Vec<FieldValue>>, Error> {
    vals.into_iter()
        .map(|vec| {
            vec.into_iter()
                .zip(fields.iter())
                .map(|(v, field)| match field.data_type {
                    FieldDataType::Float => {
                        let val = v.as_f64().ok_or_else(|| {
                            Error::UnexpectedDataType(field.name.clone(), v.clone())
                        })?;
                        Ok(FieldValue::Float(val))
                    }
                    FieldDataType::Integer => {
                        let val = v.as_i64().ok_or_else(|| {
                            Error::UnexpectedDataType(field.name.clone(), v.clone())
                        })?;
                        Ok(FieldValue::Integer(val))
                    }
                    FieldDataType::Boolean => {
                        let val = v.as_bool().ok_or_else(|| {
                            Error::UnexpectedDataType(field.name.clone(), v.clone())
                        })?;
                        Ok(FieldValue::Boolean(val))
                    }
                    FieldDataType::String => match v {
                        Value::String(s) => Ok(FieldValue::String(s)),
                        _ => Err(Error::UnexpectedDataType(field.name.clone(), v.clone())),
                    },
                })
                .collect()
        })
        .collect()
}

pub fn extract_float_value(val: &FieldValue) -> f64 {
    match val {
        FieldValue::Integer(val) => *val as f64,
        FieldValue::Float(val) => *val,
        _ => panic!("Unexpected type"),
    }
}

pub fn extract_int_value(val: &FieldValue) -> i64 {
    match val {
        FieldValue::Integer(val) => *val,
        _ => panic!("Unexpected type"),
    }
}

pub fn field_val_to_influx_val(val: &FieldValue) -> InfluxValue {
    match val {
        FieldValue::Integer(val) => InfluxValue::Integer(*val),
        FieldValue::Float(val) => InfluxValue::Float(*val),
        FieldValue::Boolean(val) => InfluxValue::Boolean(*val),
        FieldValue::String(val) => InfluxValue::String(val.to_owned()),
        // TODO: how to get rid of this clone?
        // looks like the options are:
        // 1. to move instead of borrowing, which complicates the caller side, where we don't necessary want to consume these values. or maybe even we can't move them because they are already borrowed by the downsampling function.
        // 2. patch the influx lib to have &str in the string type instead of String
        // 3. patch the influx lib to return results as InfluxValue
    }
}

pub fn to_point(v: &Vec<FieldValue>, measurement: &str, fields: &Vec<Field>) -> Point {
    let mut point = Point::new(measurement);

    let timestamp = extract_int_value(&v[0]);
    point.add_timestamp(timestamp);

    for (val, field) in v.iter().skip(1).zip(fields.iter().skip(1)) {
        let influx_val = field_val_to_influx_val(val);
        point.add_field(&field.name, influx_val);
    }

    point
}

pub fn save_points(
    client: &Client,
    retention_policy: &str,
    points: Vec<Point>,
) -> Result<(), error::Error> {
    client.write_points(
        Points::create_new(points),
        Some(Precision::Nanoseconds),
        Some(retention_policy),
    )?;

    Ok(())
}
