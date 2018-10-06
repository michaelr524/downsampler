use influx_db_client::error;
use influx_db_client::Node;
use influx_db_client::{Client, Precision};

// pass `limit` 0 to disable limit
pub fn build_query(start: i64, end: i64, limit: i64) -> String {
    format!(
        r#"select price, amount
from glukoz."glukoz-rentention-policy".trade
WHERE feed_id = 'binance'
      AND pair = 'BTCUSDT'
      AND time >= {start}
      AND time <= {end}
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
