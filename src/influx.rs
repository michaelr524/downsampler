use influx_db_client::error;
use influx_db_client::Points;
use influx_db_client::{Client, Node, Point, Precision};
use serde_json::Value;

pub fn influx_client() -> Client {
    Client::new("http://localhost:8086", "glukoz").set_authentication("root", "root")
}

// pass `limit: 0` to disable limit
pub fn build_query(start: i64, end: i64, limit: i64) -> String {
    format!(
        r#"select price, amount
from glukoz."glukoz-rentention-policy".trade
WHERE feed_id = 'binance'
      AND pair = 'BTCUSDT'
      AND time >= {start}
      AND time < {end}
      limit {limit}
      "#,
        start = start,
        end = end,
        limit = limit
    )
}

pub fn run_query(client: &Client, query: &str) -> Result<Option<Vec<Node>>, error::Error> {
    println!("{}", query);

    client.query(query, Some(Precision::Nanoseconds))
}

pub fn get_values(result: &Result<Option<Vec<Node>>, error::Error>) -> &Vec<Vec<Value>> {
    // a shorter way to do this?
    &result
        .as_ref()
        .unwrap()
        .as_ref()
        .unwrap()
        .first()
        .unwrap()
        .series
        .as_ref()
        .unwrap()
        .first()
        .as_ref()
        .unwrap()
        .values // TODO: handle errors
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
