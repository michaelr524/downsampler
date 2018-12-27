use chrono::NaiveDateTime;
use crate::downsampling::downsample_period;
use crate::influx::influx_client;
use crate::settings::Interval;
use crate::utils::time::{intervals, parse_timestamp_sec};
use crate::{cmdargs::CmdArgs, settings::Config, utils::error::print_err_and_exit};
use influx_db_client::Client;
use redis::Commands;
use redis::Connection;
use redis::PipelineCommands;
use std::collections::HashMap;
use std::ops::Sub;
use std::thread;
use std::time::Duration as StdDuration;
use string_template::Template;
use time::Duration;

const UPDATES_TABLE_NAME: &str = "downsampler_updates";
const CHECKPOINTS_TABLE_NAME: &str = "downsampler_checkpoints";

pub fn listen(_args: &CmdArgs, config: &Config) -> () {
    let influx_client = influx_client(
        &config.influxdb.url,
        &config.influxdb.db,
        &config.influxdb.username,
        &config.influxdb.pass,
    );

    let client = redis::Client::open(config.listen.redis_url.as_str())
        .unwrap_or_else(|e| print_err_and_exit(e));
    let con = client
        .get_connection()
        .unwrap_or_else(|e| print_err_and_exit(e));

    let mut checkpoints = get_checkpoints(&con); // load checkpoints just once
    println!("checkpoints: {:#?}", checkpoints);

    let measurement_template = Template::new(&config.listen.measurement_template);
    let query_template = Template::new(&config.listen.query_template);

    loop {
        let map = get_updates(&con); // get updates
        if !map.is_empty() {
            for (id, timestamp_str) in map.into_iter() {
                let end = parse_timestamp_sec(timestamp_str.as_str());
                process_period(
                    config,
                    &influx_client,
                    id.as_str(),
                    &end,
                    &query_template,
                    &measurement_template,
                    &con,
                    &mut checkpoints,
                );
            }
        } else {
            // we didn't get updates this time, sleep a bit
            thread::sleep(StdDuration::from_millis(config.listen.poll_sleep_ms));
        }
    }
}

fn process_period(
    config: &Config,
    influx_client: &Client,
    id: &str,
    period_end: &NaiveDateTime,
    query_template: &Template,
    measurement_template: &Template,
    con: &Connection,
    checkpoints: &mut HashMap<String, i64>,
) {
    for interval_period in config.downsampler.intervals.iter() {
        // check with each interval
        let key = checkpoint_key(id, interval_period);
        let period_start = calc_period_start(interval_period, period_end, checkpoints, key.as_str());

        for (_start, end) in intervals(
            period_start,
            *period_end,
            Duration::seconds(interval_period.duration_secs as i64),
        ) { // iterate the given period in duration_secs chunks
            if end >= period_start && end <= *period_end {
                println!("period_start: {:#?}, period_end: {:#?}, _start: {:#?}, end: {:#?}, interval_name: {:#?}",
                         period_start,
                         period_end,
                         _start,
                         end,
                         interval_period.name);

                let measurement_name = render_measurement_name(
                    id,
                    &measurement_template,
                    interval_period.name.as_str(),
                );
                downsample_period(
                    config,
                    &influx_client,
                    &query_template,
                    id,
                    end,
                    interval_period.duration_secs,
                    measurement_name.as_str(),
                );

                set_checkpoint(&con, checkpoints, key.as_str(), end.timestamp());

                println!("Wrote checkpoint for interval {}, {:#?}", measurement_name, end);
            }
        }
    }
}

fn calc_period_start(
    interval_period: &Interval,
    period_end: &NaiveDateTime,
    checkpoints: &mut HashMap<String, i64>,
    key: &str,
) -> NaiveDateTime {
    match checkpoints.get(key) {
        Some(ts) => NaiveDateTime::from_timestamp(*ts, 0), // start from last checkpoint if exists
        _ => {
            let period_start = period_end.sub(Duration::seconds(1)); // start from now minus 1 second if no checkpoint
            NaiveDateTime::from_timestamp(
                (period_start.timestamp() / interval_period.duration_secs as i64)
                    * interval_period.duration_secs as i64,
                0,
            ) // round to the start of an interval
        }
    }
}

fn checkpoint_key(id: &str, interval_period: &Interval) -> String {
    let key = format!("{}_{}", id, interval_period.name);
    key
}

fn render_measurement_name(
    id: &str,
    measurement_template: &Template,
    interval_period: &str,
) -> String {
    let mut m = HashMap::new();
    m.insert("id", id);
    m.insert("time_interval", interval_period);
    measurement_template.render(&m)
}

fn set_checkpoint(
    con: &Connection,
    checkpoints: &mut HashMap<String, i64>,
    id: &str,
    ts: i64,
) -> () {
    redis::cmd("HSET")
        .arg(CHECKPOINTS_TABLE_NAME)
        .arg(id)
        .arg(ts)
        .execute(con);
    checkpoints.insert(id.to_owned(), ts);
}

fn get_updates(con: &Connection) -> HashMap<String, String> {
    let (map, _): (HashMap<String, String>, i32) = redis::pipe()
        .atomic()
        .hgetall(UPDATES_TABLE_NAME)
        .del(UPDATES_TABLE_NAME)
        .query(con)
        .unwrap();

    map
}

fn get_checkpoints(con: &Connection) -> HashMap<String, i64> {
    let map: HashMap<String, String> = con.hgetall(CHECKPOINTS_TABLE_NAME).unwrap();

    map.into_iter()
        .map(|(k, ts)| (k, ts.parse::<i64>().unwrap()))
        .collect::<HashMap<String, i64>>()
}
