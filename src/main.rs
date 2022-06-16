use anyhow::{Context, Result};
use stegosaurust::{cli, run};
use structopt::StructOpt;

fn main() -> Result<()> {
    let opt = cli::Opt::from_args();
    run(opt).context("failed to run steganography")?;
    Ok(())
}
