use stegosaurust::{cli, run};
use structopt::StructOpt;

fn main() {
    let opt = cli::Opt::from_args();
    if let Err(e) = opt.validate() {
        eprintln!("Invalid arguments: {:?}", e);
        std::process::exit(1);
    }
    if let Err(e) = run(opt) {
        eprintln!("{:?}", e);
        std::process::exit(1);
    }
}
