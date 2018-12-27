#[macro_use]
extern crate serde_derive;

mod cmdargs;
mod downsampler;
mod downsampling;
mod influx;
mod listen;
mod lttb;
mod settings;
mod splitter;
mod utils;

use crate::{
    cmdargs::{parse_args, print_args_info, CmdArgs},
    downsampler::downsample,
    listen::listen,
    settings::config_from_file,
    splitter::split,
    utils::error::print_err_and_exit,
};

fn main() {
    let args = parse_args().unwrap_or_else(|e| print_err_and_exit(e));
    print_args_info(&args);

    let settings = config_from_file("config").unwrap_or_else(|e| print_err_and_exit(e));

    match args {
        CmdArgs::Downsample(period) => downsample(&period, &settings),
        CmdArgs::Split(period) => split(&period, &settings),
        CmdArgs::Listen => listen(&args, &settings),
    };
}
