extern crate chrono;
#[macro_use]
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
extern crate config;
extern crate rayon;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate string_template;

mod cmdargs;
mod downsampler;
mod influx;
mod lttb;
mod settings;
mod splitter;
mod utils;

use cmdargs::{parse_args, print_args_info, Command};
use downsampler::downsample;
use settings::config_from_file;
use splitter::split;
use utils::error::print_err_and_exit;

fn main() {
    let args = parse_args().unwrap_or_else(|e| print_err_and_exit(e));
    print_args_info(&args);

    let settings = config_from_file("config").unwrap_or_else(|e| print_err_and_exit(e));

    match args.command {
        Command::Downsample => downsample(&args, &settings),
        Command::Split => split(&args, &settings),
    };
}
