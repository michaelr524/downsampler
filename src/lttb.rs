pub trait DataPoint {
    fn get_x(&self) -> f64;
    fn get_y(&self) -> f64;
}

// copied from https://github.com/jeromefroe/lttb-rs/blob/master/src/lib.rs
// modified to be generic and return references to the original data
// instead of copying
// TODO: verify implementation is correct
pub fn lttb_downsample<T: DataPoint>(data: &Vec<T>, threshold: usize) -> Option<Vec<&T>> {
    if threshold >= data.len() || threshold == 0 {
        return None;
    }

    let mut sampled = Vec::with_capacity(threshold);

    // Bucket size. Leave room for start and end data points.
    let every = ((data.len() - 2) as f64) / ((threshold - 2) as f64);

    // Initially a is the first point in the triangle.
    let mut a = 0;

    // Always add the first point.
    sampled.push(&data[a]);

    for i in 0..threshold - 2 {
        // Calculate point average for next bucket (containing c).
        let mut avg_x = 0f64;
        let mut avg_y = 0f64;

        let avg_range_start = (((i + 1) as f64) * every) as usize + 1;

        let mut end = (((i + 2) as f64) * every) as usize + 1;
        if end >= data.len() {
            end = data.len();
        }
        let avg_range_end = end;

        let avg_range_length = (avg_range_end - avg_range_start) as f64;

        for i in 0..(avg_range_end - avg_range_start) {
            let idx = (avg_range_start + i) as usize;
            avg_x += data[idx].get_x();
            avg_y += data[idx].get_y();
        }
        avg_x /= avg_range_length;
        avg_y /= avg_range_length;

        // Get the range for this bucket.
        let range_offs = ((i as f64) * every) as usize + 1;
        let range_to = (((i + 1) as f64) * every) as usize + 1;

        // Point a.
        let point_a_x = data[a].get_x();
        let point_a_y = data[a].get_y();

        let mut max_area = -1f64;
        let mut next_a = range_offs;
        for i in 0..(range_to - range_offs) {
            let idx = (range_offs + i) as usize;

            // Calculate triangle area over three buckets.
            let area = ((point_a_x - avg_x) * (data[idx].get_y() - point_a_y)
                - (point_a_x - data[idx].get_x()) * (avg_y - point_a_y))
                .abs() * 0.5;
            if area > max_area {
                max_area = area;
                next_a = idx; // Next a is this b.
            }
        }

        sampled.push(&data[next_a]); // Pick this point from the bucket.
        a = next_a; // This a is the next a (chosen b).
    }

    // Always add the last point.
    sampled.push(&data[data.len() - 1]);

    Some(sampled)
}
