use stegosaurust::{Opt,run};
use std::process::exit;
use structopt::StructOpt;

fn main() {
    let opt = Opt::from_args();
    if let Err(e) = run(opt) {
        eprintln!("{}", e);
        exit(1);
    }
}
