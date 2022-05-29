use stegosaurust::{run,cli};
use structopt::StructOpt;
use anyhow::{Context,Result};

fn main() -> Result<()> {
    let opt = cli::Opt::from_args();
    opt.validate()?;
    run(opt).context("failed to run steganography")?;
    Ok(())
}
