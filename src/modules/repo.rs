use crate::utils::args::RepoCommands;
use crate::utils::error::Error;
pub fn repo(args: RepoCommands) -> Result<(), Error> {
    println!("{:?}", args);
    Ok(())
}
