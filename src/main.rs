#![allow(dead_code)]
mod database;
mod interface;

use crate::interface::Args;
use crate::interface::Mode;

use ansi_term::Style;
use structopt::StructOpt;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
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
            let local_database = database::local::read_local_database()?;
            for package in local_database.iter() {
                let style = Style::new().bold();
                println!(
                    "{} {}",
                    style.paint(package.desc.name.as_str()),
                    style
                        .fg(ansi_term::Color::Green)
                        .paint(package.desc.version.as_str())
                );
            }
            Ok(())
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
