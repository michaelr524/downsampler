use crate::cmdargs::TimePeriod;
use crate::downsampling::is_downsampling_interval;
use crate::{
    downsampling::downsample_period, influx::influx_client, settings::Config,
    utils::time::intervals,
};
use rayon::prelude::*;
use std::collections::HashMap;
use string_template::Template;
use time::Duration;

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

pub fn downsample(args: &TimePeriod, config: &Config) -> () {
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

        for (start, _end) in intervals(args.start, args.end, Duration::seconds(1)) {
            for interval_period in config.downsampler.intervals.iter() {
                if is_downsampling_interval(&start, interval_period) {
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
