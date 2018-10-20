use chrono::NaiveDateTime;
use influx_db_client::error;
use influx_db_client::Points;
use influx_db_client::{Client, Node, Point, Precision, Value as InfluxValue};
use serde_json::Value;

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "Failed to access InfluxDB: Error: {:?}", _0)]
    InfluxDbAccessError(error::Error),
    #[fail(display = "InfluxDB query didn't return a result.",)]
    NoResult,
    #[fail(display = "Valid timestamp wasn't found InfluxDB result record.",)]
    CouldNotFindTimestamp,
    #[fail(display = "InfluxDB returned a field with null value.",)]
    UnexpectedNullFieldValue,
    #[fail(display = "InfluxDB returned a field with a data type that we didn't expect.",)]
    UnexpectedDataType,
    #[fail(display = "InfluxDB returned a numeric field with a data type that we didn't expect.",)]
    UnexpectedNumberDataType,
}

pub struct SeriesResult {
    pub values: Vec<Vec<Value>>,
    pub columns: Vec<String>,
}

pub fn influx_client() -> Client {
    Client::new("http://localhost:8086", "glukoz").set_authentication("root", "root")
}

// pass `limit: 0` to disable limit
pub fn build_query(pair: &str, start: i64, end: i64, limit: i64) -> String {
    format!(
        r#"select price, amount
from glukoz."glukoz-rentention-policy".trade
WHERE feed_id = 'binance'
      AND pair = '{pair}'
      AND time >= {start}
      AND time < {end}
      limit {limit}
      "#,
        start = start,
        end = end,
        limit = limit,
        pair = pair
    )
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

pub fn get_range(
    client: &Client,
    pair_name: &str,
    start: NaiveDateTime,
    end: NaiveDateTime,
) -> Result<SeriesResult, Error> {
    let query = build_query(pair_name, start.timestamp_nanos(), end.timestamp_nanos(), 0);
    let res = run_query(&client, &query);
    first_series_from_result(res)
}

pub fn extract_timestamp(record: &Vec<Value>) -> Result<i64, Error> {
    Ok(record
        .first()
        .ok_or_else(|| Error::CouldNotFindTimestamp)?
        .as_i64()
        .ok_or_else(|| Error::CouldNotFindTimestamp)?)
}

pub fn json_val_to_influx_val(val: &Value) -> Result<InfluxValue, Error> {
    let influx_val = match val {
        Value::Null => return Err(Error::UnexpectedNullFieldValue),
        Value::Bool(val) => InfluxValue::Boolean(*val),
        Value::Number(val) => {
            // TODO: handle generically. currently converts all numbers to floats, because some
            // of the values in float columns returned as integers (maybe because of JSON, or maybe
            // they are written using the wrong type)
//            if val.is_f64() {
//                // safe because we've checked with .is_f64()
//                InfluxValue::Float(val.as_f64().unwrap())
//            } else if val.is_i64() {
//                // safe because we've checked with .is_i64()
//                InfluxValue::Integer(val.as_i64().unwrap())
            if val.is_f64() || val.is_i64() {
                // safe because we've checked with .is_i64() and is_f64()
                InfluxValue::Float(val.as_f64().unwrap())
            } else {
                return Err(Error::UnexpectedNumberDataType);
            }
        }
        Value::String(val) => InfluxValue::String(val.to_owned()),
        _ => return Err(Error::UnexpectedDataType),
    };

    Ok(influx_val)
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
