use cmd_arg::cmd_arg;
use ipak::utils::color::colorize::*;
use ipak::modules::pkg::AuthorAboutData;
use ipak::utils::files::{dir_creation, file_creation};
use ipak::{
    dprintln,
    utils::{generate_email_address, shell::username},
};
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::fmt;
use std::fs;
#[derive(Serialize, Deserialize)]
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
        writeln!(
            f,
            "{}: {}",
            "Author Name".bold(),
            self.author_name
        )?;
        writeln!(
            f,
            "{}: {}",
            "Author Email".bold(),
            self.author_email
        )
    }
}

pub fn init(
    args: Vec<&cmd_arg::Option>,
) -> Result<(), std::io::Error> {
    let mut opts = ServerRepoInitOptions::default();
    for arg in args {
        match arg.opt_str.as_str() {
            "--name" => {
                if arg.opt_values.len() == 1 {
                    opts.author_name = arg
                        .opt_values
                        .first()
                        .unwrap()
                        .to_owned();
                }
            }
            "--email" => {
                if arg.opt_values.len() == 1 {
                    opts.author_email = arg
                        .opt_values
                        .first()
                        .unwrap()
                        .to_owned();
                }
            }
            _ => continue,
        }
    }
    dprintln!("{}", opts);
    server_initation(opts)?;
    Ok(())
}

fn server_initation(
    opts: ServerRepoInitOptions,
) -> Result<(), std::io::Error> {
    let entries = fs::read_dir(".")?;
    if entries.count() > 0 {
        eprintln!(
            "{}",
            "Error: Current directory is not empty.".red()
        );
        return Err(std::io::Error::other(
            "Current directory is not empty",
        ));
    }
    struct SetUpData {
        from: String,
        to: String,
    }
    let setup_list = [
        SetUpData {
            from: {
                let target_data = AuthorAboutData {
                    name: opts.author_name.clone(),
                    email: opts.author_email.clone(),
                };
                serde_yaml::to_string(&target_data).map_err(
                    |e| -> std::io::Error {
                        std::io::Error::other(
                            e,
                        )
                    },
                )
            }?,
            to: "ipm/repo.yaml".to_owned(),
        },
        SetUpData {
            from: "".to_owned(),
            to: "projects".to_owned(),
        },
        SetUpData { from: "".to_owned(), to: "out".to_owned() },
    ];
    for setup_data in setup_list {
        if setup_data.from.is_empty() {
            dir_creation(&setup_data.to)?;
        } else {
            file_creation(&setup_data.to, &setup_data.from)?;
        }
    }
    Ok(())
}
