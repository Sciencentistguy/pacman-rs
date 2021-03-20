#![allow(dead_code)]
mod interface;
mod database;

use crate::interface::Args;
use crate::interface::Mode;

use structopt::StructOpt;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() {
    println!("Hello, world!");
    let args = Args::from_args();
    let mode = args.parse_mode();

    match mode {
        Mode::Database => {
            unimplemented!()
        }
        Mode::Files => {
            unimplemented!()
        }
        Mode::Query => {
            unimplemented!()
        }
        Mode::Remove => {
            unimplemented!()
        }
        Mode::Sync => {
            unimplemented!()
        }
        Mode::Deptest => {
            unimplemented!()
        }
        Mode::Upgrade => {
            unimplemented!()
        }
    }
}
