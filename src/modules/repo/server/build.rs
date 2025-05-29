use super::metadata;
use cmd_arg::cmd_arg;
use ipak::modules::project;
use ipak::utils::files::file_creation;
use std::env;
pub fn build() -> Result<(), std::io::Error> {
    let repo_metadata = metadata::metadata()?;
    let target_path = metadata::get_dir()?;
    // Clean up {target_path}/out directory
    let out_dir = target_path.join("out");
    if out_dir.exists() {
        std::fs::remove_dir_all(&out_dir)?;
    }
    std::fs::create_dir_all(&out_dir)?;

    // Iterate over all directories in {target_path}/projects and build each project
    let projects_dir = target_path.join("projects");
    for entry in std::fs::read_dir(&projects_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let original_dir = env::current_dir()?;
            env::set_current_dir(&path)?;
            let build_result = project::project(vec![
                &cmd_arg::Option {
                    opt_str: "build".to_owned(),
                    opt_values: vec![],
                    opt_type: cmd_arg::OptionType::Simple,
                },
                &cmd_arg::Option {
                    opt_str: "--release".to_owned(),
                    opt_values: vec![],
                    opt_type: cmd_arg::OptionType::LongOpt,
                },
            ]);
            let package_result =
                project::project(vec![&cmd_arg::Option {
                    opt_str: "package".to_owned(),
                    opt_values: vec![],
                    opt_type: cmd_arg::OptionType::Simple,
                }]);
            env::set_current_dir(&original_dir)?;
            build_result?;
            package_result?;
            let package_src = path.join("ipak/package");
            let package_dst = out_dir.join("packages");
            if package_src.exists() {
                std::fs::create_dir_all(&package_dst)?;
                for pkg_entry in std::fs::read_dir(&package_src)? {
                    let pkg_entry = pkg_entry?;
                    let pkg_path = pkg_entry.path();
                    if pkg_path.is_file() {
                        let file_name = pkg_path.file_name().unwrap();
                        let dst_path = package_dst.join(file_name);
                        std::fs::copy(&pkg_path, &dst_path)?;
                    }
                }
            }
        }
    }
    let repo_metadata = serde_yaml::to_string(&repo_metadata)
        .map_err(|e| -> std::io::Error {
            std::io::Error::new(std::io::ErrorKind::Other, e)
        })?;
    file_creation("out/repo.yaml", &repo_metadata)?;
    Ok(())
}
