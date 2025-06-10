use std::path::PathBuf;

use cmd_arg::cmd_arg;
use ipak::dprintln;

pub fn convert(
    args: Vec<&cmd_arg::Option>,
) -> Result<(), std::io::Error> {
    let mut path_from = String::new();
    let mut path_to = String::new();
    for arg in args {
        match arg.opt_str.as_str() {
            "--from" => {
                if let Some(val) = arg.opt_values.get(0) {
                    path_from = val.clone();
                }
            }
            "--to" => {
                if let Some(val) = arg.opt_values.get(0) {
                    path_to = val.clone();
                }
            }
            _ => continue,
        }
    }
    let path_from = PathBuf::from(path_from);
    let path_to = PathBuf::from(path_to);
    dprintln!(
        "Converting package from: {} to: {}",
        path_from.display(),
        path_to.display()
    );
    Ok(())
}
