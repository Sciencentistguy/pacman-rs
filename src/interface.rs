pub use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "pacman-rs")]
pub struct Args {
    /// Database mode
    #[structopt(long, short = "D", group = "mode")]
    pub database: bool,
    /// Files mode
    #[structopt(long, short = "F", group = "mode")]
    pub files: bool,
    /// Query mode
    #[structopt(long, short = "Q", group = "mode")]
    pub query: bool,
    /// Remove mode
    #[structopt(long, short = "R", group = "mode")]
    pub remove: bool,
    /// Sync mode
    #[structopt(long, short = "S", group = "mode")]
    pub sync: bool,
    /// Deptest mode
    #[structopt(long, short = "T", group = "mode")]
    pub deptest: bool,
    /// Upgrade mode
    #[structopt(long, short = "U", group = "mode")]
    pub upgrade: bool,
}

impl Args {
    pub fn parse_mode(&self) -> Mode {
        if self.database {
            Mode::Database
        } else if self.files {
            Mode::Files
        } else if self.query {
            Mode::Query
        } else if self.remove {
            Mode::Remove
        } else if self.sync {
            Mode::Sync
        } else if self.deptest {
            Mode::Deptest
        } else if self.upgrade {
            Mode::Upgrade
        } else {
            panic!("Error: No operation mode provided");
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mode {
    Database,
    Files,
    Query,
    Remove,
    Sync,
    Deptest,
    Upgrade,
}
