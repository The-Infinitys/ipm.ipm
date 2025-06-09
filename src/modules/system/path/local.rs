use ipak::utils::shell;
use std::env;
use std::path::PathBuf;
pub fn repo_list_path() -> PathBuf {
    ipm_dir().join("repos.repo")
}
fn home_dir() -> PathBuf {
    PathBuf::from(
        env::var("HOME")
            .unwrap_or(format!("/home/{}", shell::username())),
    )
}
fn ipm_dir() -> PathBuf {
    home_dir().join(".ipm")
}
