use chrono::format::ParseError;
use chrono::offset::TimeZone;
use chrono::{NaiveDateTime, Utc};
use clap::ArgGroup;
use clap::{App, Arg};
use humantime::{parse_duration as human_parse_duration, DurationError};
use time::{Duration, OutOfRangeError};
use utils::time::truncate_seconds;

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(
        display = "Failed to parse datetime from argument: {:?}, Error: {:?}",
        datetime,
        inner
    )]
    DateParseError {
        datetime: Option<String>,
        inner: Option<ParseError>,
    },
    #[fail(
        display = "Failed to parse duration from argument: {:?}, Error: {:?}",
        duration,
        inner
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

pub struct Settings {
    pub raw_start: NaiveDateTime,
    pub raw_end: NaiveDateTime,
    pub start: NaiveDateTime,
    pub end: NaiveDateTime,
}

fn args_definitions<'a, 'b>() -> App<'a, 'b> {
    App::new("Downsampler")
        .version("1.0")
        .author("Michael Ravits. <michael@xlucidity.com>")
        .about("Does awesome things")
        .arg(
            Arg::with_name("start")
                .short("s")
                .long("start")
                .value_name("DATETIME")
                .help("Start time e.g 2018-10-10 10:10:10")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("end")
                .short("e")
                .long("end")
                .value_name("DATETIME")
                .help("End time e.g 2018-11-11 11:11:11")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("duration")
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
                .takes_value(true),
        ).group(
        ArgGroup::with_name("period_end")
            .required(true)
            .args(&["end", "duration"])
    )

    // TODO::::
    //    MOVE DOWNSAMPLE INTO SUB COMMAND
    //    ADD ANOTHER TOP COMMAND FOR SPLITTING TIMESERIES
}

pub fn parse_args() -> Result<Settings, Error> {
    let args = args_definitions().get_matches();

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

    Ok(Settings {
        start,
        end,
        raw_start,
        raw_end,
    })
}

pub fn print_settings_info(settings: &Settings) {
    println!("Period {:?} - {:?}", settings.raw_start, settings.raw_end);
    println!("Period truncated {:?} - {:?}", settings.start, settings.end);
    println!(
        "Period in nanos {:?} - {:?}",
        settings.start.timestamp_nanos(),
        settings.end.timestamp_nanos()
    );
}

fn parse_duration(date_string: Option<&str>) -> Result<Duration, Error> {
    let duration_str = date_string.ok_or(Error::DurationParseError {
        inner: None,
        duration: date_string.map_or(None, |s| Some(s.to_string())),
    })?;

    let duration_std = human_parse_duration(duration_str).map_err(|e| Error::DurationParseError {
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
