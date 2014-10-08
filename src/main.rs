extern crate getopts;
extern crate in_order;

#[cfg(not(test))]
use std::os;
#[cfg(not(test))]
use in_order::config::{Config, Do, Undo};
#[cfg(not(test))]
use getopts::{optopt, optflag, getopts, usage, OptGroup};

#[cfg(not(test))]
fn print_usage(opts: &[OptGroup]) {
    println!("{}", usage("USAGE: in-order [do|undo]", opts));
}

#[cfg(not(test))]
fn main() {
    let args: Vec<String> = os::args();

    let opts = [
        optopt("c", "config", "specify configuration file", "FILE"),
        optflag("h", "help", "print this help menu")
    ];

    let matches = match getopts(args.tail(), opts) {
        Ok(m) => m,
        Err(error) => {
            println!("{}", error);
            return
        }
    };

    if matches.opt_present("h") {
        print_usage(opts);
        return;
    }

    let direction =
        if matches.free.len() > 0 && matches.free[0].as_slice() == "undo" {
            Undo
        } else {
            Do
        };

    let mut config: Config = match Config::read(matches.opt_str("c")) {
        Ok(config) => config,
        Err(error) => {
            println!("{}", error);
            return
        }
    };

    config.perform(direction);
}
