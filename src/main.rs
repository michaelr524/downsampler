#[macro_use]
extern crate serde_derive;

mod cmdargs;
mod downsampler;
mod influx;
mod lttb;
mod settings;
mod splitter;
mod utils;

use crate::{
    cmdargs::{parse_args, print_args_info, Command},
    downsampler::downsample,
    settings::config_from_file,
    splitter::split,
    utils::error::print_err_and_exit,
};

fn main() {
    let args = parse_args().unwrap_or_else(|e| print_err_and_exit(e));
    print_args_info(&args);

    let settings = config_from_file("config").unwrap_or_else(|e| print_err_and_exit(e));

    match args.command {
        Command::Downsample => downsample(&args, &settings),
        Command::Split => split(&args, &settings),
    };
}
