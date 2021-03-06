use chrono::{format::ParseError, offset::TimeZone, NaiveDateTime, Utc};
use clap::ArgMatches;
use clap::{crate_version, App, Arg, ArgGroup, SubCommand};
use crate::utils::time::truncate_seconds;
use failure_derive::Fail;
use humantime::{parse_duration as human_parse_duration, DurationError};
use time::{Duration, OutOfRangeError};

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "No command has been specified")]
    CommandMissing,
    #[fail(
        display = "Failed to parse datetime from argument: {:?}, Error: {:?}",
        datetime, inner
    )]
    DateParseError {
        datetime: Option<String>,
        inner: Option<ParseError>,
    },
    #[fail(
        display = "Failed to parse duration from argument: {:?}, Error: {:?}",
        duration, inner
    )]
    DurationParseError {
        duration: Option<String>,
        inner: Option<DurationError>,
    },
    #[fail(
        display = "Invalid `start` argument passed. It should have this format: '2018-10-10 10:10:10'. Error: {:?}",
        _0
    )]
    InvalidStartArgument(Box<Error>),
    #[fail(
        display = "Invalid `end` argument passed. It should have this format: '2018-10-10 10:10:10'. Error: {:?}",
        _0
    )]
    InvalidEndArgument(Box<Error>),
    #[fail(
        display = "Invalid `duration` argument passed. It should have this format: '1hour 12min 5s'. Error: {:?}",
        _0
    )]
    InvalidDurationArgument(Box<Error>),
    #[fail(
        display = "Invalid `duration` argument passed. It exceeds the supported duration length. Error: {:?}",
        _0
    )]
    DurationTooLong(OutOfRangeError),
}

pub struct TimePeriod {
    pub raw_start: NaiveDateTime,
    pub raw_end: NaiveDateTime,
    pub start: NaiveDateTime,
    pub end: NaiveDateTime,
}

pub enum CmdArgs {
    Downsample(TimePeriod),
    Split(TimePeriod),
    Listen,
}

fn args_definitions<'a, 'b>() -> App<'a, 'b> {
    let start_arg = Arg::with_name("start")
        .short("s")
        .long("start")
        .value_name("DATETIME")
        .help("Start time e.g 2018-10-10 10:10:10")
        .required(true)
        .takes_value(true);

    let end_arg = Arg::with_name("end")
        .short("e")
        .long("end")
        .value_name("DATETIME")
        .help("End time e.g 2018-11-11 11:11:11")
        .takes_value(true);

    let duration_arg = Arg::with_name("duration")
        .short("d")
        .long("duration")
        .value_name("TIME")
        .help(r#"Duration e.g '1hour 12min 5s'
The duration object is a concatenation of time spans. Where each time span is an integer number and a suffix. Supported suffixes:

nsec, ns -- microseconds
usec, us -- microseconds
msec, ms -- milliseconds
seconds, second, sec, s
minutes, minute, min, m
hours, hour, hr, h
days, day, d
weeks, week, w
months, month, M -- defined as 30.44 days
years, year, y -- defined as 365.25 days
            "#)
        .takes_value(true);

    let period_end_group = ArgGroup::with_name("period_end")
        .required(true)
        .args(&["end", "duration"]);

    App::new("Downsampler")
        .version(crate_version!())
        .author("Michael Ravits <michael@xlucidity.com>")
        .about("Utilities for transforming InfluxDB time series data")
        .bin_name("downsampler")
        .subcommand(
            SubCommand::with_name("split")
                .about("Splits single measurement with many series into many separate measurements")
                .arg(start_arg.clone())
                .arg(end_arg.clone())
                .arg(duration_arg.clone())
                .group(period_end_group.clone()),
        )
        .subcommand(
            SubCommand::with_name("downsample")
                .about("Creates downsampled series from a series")
                .arg(start_arg.clone())
                .arg(end_arg.clone())
                .arg(duration_arg.clone())
                .group(period_end_group.clone()),
        )
        .subcommand(SubCommand::with_name("listen").about("Continuous downsampling"))
}

pub fn parse_args() -> Result<CmdArgs, Error> {
    let args = args_definitions().get_matches();

    match args.subcommand() {
        ("downsample", Some(subcommand)) => {
            let time_period = parse_time_period(subcommand)?;
            Ok(CmdArgs::Downsample(time_period))
        }
        ("split", Some(subcommand)) => {
            let time_period = parse_time_period(subcommand)?;
            Ok(CmdArgs::Split(time_period))
        }
        ("listen", Some(_)) => Ok(CmdArgs::Listen),
        _ => {
            args_definitions().print_help().unwrap();
            return Err(Error::CommandMissing);
        }
    }
}

fn parse_time_period(args: &ArgMatches) -> Result<TimePeriod, Error> {
    let raw_start = parse_datetime(args.value_of("start"))
        .map_err(|e| Error::InvalidStartArgument(Box::new(e)))?;
    let raw_end = if args.is_present("duration") {
        let duration = parse_duration(args.value_of("duration"))
            .map_err(|e| Error::InvalidDurationArgument(Box::new(e)))?;
        raw_start + duration
    } else {
        let datetime = parse_datetime(args.value_of("end"))
            .map_err(|e| Error::InvalidEndArgument(Box::new(e)))?;
        datetime
    };
    let start = truncate_seconds(raw_start);
    let end = truncate_seconds(raw_end);

    Ok(TimePeriod {
        start,
        end,
        raw_start,
        raw_end,
    })
}

pub fn print_args_info(settings: &CmdArgs) {
    if let Some(time_period) = match settings {
        CmdArgs::Downsample(time_period) => Some(time_period),
        CmdArgs::Split(time_period) => Some(time_period),
        CmdArgs::Listen => None,
    } {
        println!(
            "Period {:?} - {:?}",
            time_period.raw_start, time_period.raw_end
        );
        println!(
            "Period truncated {:?} - {:?}",
            time_period.start, time_period.end
        );
        println!(
            "Period in nanos {:?} - {:?}",
            time_period.start.timestamp_nanos(),
            time_period.end.timestamp_nanos()
        );
    }
}

fn parse_duration(date_string: Option<&str>) -> Result<Duration, Error> {
    let duration_str = date_string.ok_or(Error::DurationParseError {
        inner: None,
        duration: date_string.map_or(None, |s| Some(s.to_string())),
    })?;

    let duration_std =
        human_parse_duration(duration_str).map_err(|e| Error::DurationParseError {
            inner: Some(e),
            duration: date_string.map_or(None, |s| Some(s.to_string())),
        })?;

    Duration::from_std(duration_std).map_err(|e| Error::DurationTooLong(e))
}

fn parse_datetime(date_string: Option<&str>) -> Result<NaiveDateTime, Error> {
    let date_str = date_string.ok_or(Error::DateParseError {
        inner: None,
        datetime: date_string.map_or(None, |s| Some(s.to_string())),
    })?;

    let date_time = Utc
        .datetime_from_str(date_str, "%Y-%m-%d %H:%M:%S")
        .map_err(|e| Error::DateParseError {
            inner: Some(e),
            datetime: date_string.map_or(None, |s| Some(s.to_string())),
        })?;

    Ok(date_time.naive_utc())
}
