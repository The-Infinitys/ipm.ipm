use crate::modules::repo::{PackageMetaData, RepoData};
use ipak::modules::pkg::AuthorAboutData;
use ipak::modules::project;
use ipak::utils::files::is_file_exists;

use ipak::dprintln;
use std::{env, io, path::PathBuf}; // io::Error をインポート

pub fn get_dir() -> Result<PathBuf, io::Error> {
    let mut current_path = env::current_dir()?; // Result を直接扱う
    loop {
        let metadata_path = current_path.join("ipm/repo.yaml");
        dprintln!("{}", metadata_path.display()); // .to_str().unwrap() を避ける
        if is_file_exists(metadata_path.to_str().ok_or_else(
            || {
                // .to_str() の失敗を考慮
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Invalid path characters",
                )
            },
        )?) {
            return Ok(current_path);
        } else {
            dprintln!(
                "Not found repo.yaml in {}",
                current_path.display()
            );
            if let Some(parent) = current_path.parent() {
                current_path = parent.to_owned(); // 親ディレクトリに移動
            } else {
                // ルートディレクトリに到達し、project.yaml が見つからなかった場合
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "project.yaml not found in current or parent directories",
                ));
            }
        }
    }
}
pub fn get_path() -> Result<PathBuf, io::Error> {
    get_dir().map(|dir| dir.join("ipm/repo.yaml"))
}
pub fn metadata() -> Result<RepoData, io::Error> {
    let metadata_path = get_path()?; // get_path() のエラーを伝播
    let read_data = std::fs::read_to_string(&metadata_path)
        .map_err(|e| {
            io::Error::new(
                e.kind(),
                format!(
                    "Failed to read {}: {}",
                    metadata_path.display(),
                    e
                ),
            )
        })?;

    let author_about_data =
        serde_yaml::from_str::<AuthorAboutData>(&read_data)
            .map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "Failed to parse {}: {}",
                        metadata_path.display(),
                        e
                    ),
                )
            })?;
    let last_modified = chrono::Local::now();
    let mut projects: Vec<PackageMetaData> = vec![];
    let projects_dir = get_dir()?.join("projects");
    if projects_dir.is_dir() {
        for entry in std::fs::read_dir(&projects_dir)? {
            let entry = entry?;
            if entry.path().is_dir() {
                // 現在のカレントディレクトリを保存
                let original_dir = env::current_dir()?;
                // プロジェクトディレクトリに移動
                env::set_current_dir(entry.path())?;
                // メタデータ取得を試みる
                let project_metadata_result =
                    project::metadata::metadata();
                // 元のディレクトリに戻す
                env::set_current_dir(&original_dir)?;
                // 成功したか確認
                match project_metadata_result {
                    Ok(project_data) => {
                        dprintln!(
                            "Successfully got metadata for {}",
                            entry.file_name().to_string_lossy()
                        );
                        let url = format!(
                            "/packages/{}-{}.ipak",
                            &project_data.about.package.name,
                            &project_data.about.package.version
                        );
                        projects.push(PackageMetaData {
                            url: url,
                            last_modified,
                            info: project_data,
                        });
                    }

                    Err(e) => eprintln!(
                        "Failed to get metadata for {}: {}",
                        entry.file_name().to_string_lossy(),
                        e
                    ),
                }
                dprintln!(
                    "{}",
                    entry.file_name().to_string_lossy()
                );
            }
        }
    }
    Ok(RepoData {
        author: author_about_data,
        last_modified,
        packages: projects,
    })
}
pub fn show_metadata() -> Result<(), io::Error> {
    let package_data = metadata()?; // metadata() のエラーを伝播
    println!("{}", package_data);
    Ok(())
}
