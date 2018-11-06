use failure::Fail;
use std::process::exit;

pub fn print_err_and_exit<T: Fail>(e: T) -> ! {
    println!("\n\nError: {:?}", e);
    exit(-1)
}
