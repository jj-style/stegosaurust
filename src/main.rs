use stegosaurust::{cli, run};
use structopt::StructOpt;

fn main() {
    let opt = cli::Opt::from_args();
    if let Err(e) = run(opt) {
        eprintln!("Error: {:?}", e);
        std::process::exit(1);
    }
}
