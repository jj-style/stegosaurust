use stegosaurust::{cli, run};
use structopt::StructOpt;

extern crate env_logger;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let opt = cli::Opt::from_args();
    if let Err(e) = run(opt) {
        eprintln!("{:?}", e);
        std::process::exit(1);
    }
}
