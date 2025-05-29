use cmd_arg::cmd_arg;
use colored::Colorize;
use ipak::utils::{generate_email_address, shell::username};
use std::fmt;
struct ServerRepoInitOptions {
    author_name: String,
    author_email: String,
}
impl Default for ServerRepoInitOptions {
    fn default() -> Self {
        Self {
            author_name: username(),
            author_email: generate_email_address(),
        }
    }
}

impl fmt::Display for ServerRepoInitOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}: {}", "Author Name".bold(), self.author_name)?;
        writeln!(f, "{}: {}", "Author Email".bold(), self.author_email)
    }
}

pub fn init(args: Vec<&cmd_arg::Option>) -> Result<(), std::io::Error> {
    let mut opts = ServerRepoInitOptions::default();
    for arg in args {
        match arg.opt_str.as_str() {
            "--name" => {
                if arg.opt_values.len() == 1 {
                    opts.author_name =
                        arg.opt_values.first().unwrap().to_owned();
                }
            }
            "--email" => {
                if arg.opt_values.len() == 1 {
                    opts.author_email =
                        arg.opt_values.first().unwrap().to_owned();
                }
            }
            _ => continue,
        }
    }
    println!("{}", opts);
    Ok(())
}
