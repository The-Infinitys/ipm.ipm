use clap::Parser;
use ipm::utils::args::Args;
use ipm::utils::args::CommandExecution;
use ipm::utils::error::Error;
fn main() -> Result<(), Error> {
    let args = Args::parse();
    args.command.exec()
}
