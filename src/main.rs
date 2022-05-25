use stegosaurust::{Opt,run};
use structopt::StructOpt;
use anyhow::{Context,Result};

fn main() -> Result<()> {
    let opt = Opt::from_args();
    run(opt).context("failed to run steganography")?;
    Ok(())
}
