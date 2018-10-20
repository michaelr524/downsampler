extern crate chrono;
extern crate clap;
extern crate env_logger;
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate humantime;
extern crate time;
//#[macro_use]
extern crate influx_db_client;
//#[macro_use]
extern crate rayon;
extern crate serde_json;

mod influx;
mod lttb;
mod settings;
mod splitter;
mod trade;
mod utils;

use settings::parse_args;
use settings::print_settings_info;
use splitter::split;
use std::process::exit;

fn main() {
    let settings = parse_args().unwrap_or_else(|e| {
        println!("Error: {}", e);
        exit(-1)
    });

    print_settings_info(&settings);

    split(&settings);
}
