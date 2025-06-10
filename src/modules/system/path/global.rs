use std::path::PathBuf;
pub fn repo_list_path() -> PathBuf {
    PathBuf::from("/usr/ipm/repos.repo")
}

pub fn cache_dir() -> PathBuf {
    PathBuf::from("/var/cache/ipm")
}
