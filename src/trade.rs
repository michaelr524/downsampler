use influx_db_client::{Point, Value};
use lttb::DataPoint;

#[derive(Debug, PartialEq)]
pub struct Trade {
    pub price: f64,
    pub amount: f64,
    pub timestamp: i64,
}

impl DataPoint for Trade {
    fn get_x(&self) -> f64 {
        self.timestamp as f64
    }

    fn get_y(&self) -> f64 {
        self.price
    }
}

impl Trade {
    pub fn to_point(&self, measurement: &str) -> Point {
        let mut point = Point::new(measurement);

        point
            .add_timestamp(self.timestamp)
            .add_field("amount", Value::Float(self.amount))
            .add_field("price", Value::Float(self.price));

        point
    }
}
