use ipak::dprintln;
use ipak::modules::pkg::PackageData;
use ipak::utils::archive::extract_archive;
use serde_yaml;
use std::fmt::{self, Display};
use std::fs;
use std::path::Path;
mod templates;
fn is_debian<P: AsRef<Path>>(dir: P) -> bool {
    let dir = dir.as_ref();
    let debian_binary = dir.join("debian-binary");
    let control_tar = std::fs::read_dir(dir)
        .map(|entries| {
            entries.filter_map(|e| e.ok()).any(|e| {
                let name = e.file_name();
                let name = name.to_string_lossy();
                name.starts_with("control.tar.")
            })
        })
        .unwrap_or(false);
    let data_tar = std::fs::read_dir(dir)
        .map(|entries| {
            entries.filter_map(|e| e.ok()).any(|e| {
                let name = e.file_name();
                let name = name.to_string_lossy();
                name.starts_with("data.tar.")
            })
        })
        .unwrap_or(false);
    debian_binary.is_file() && control_tar && data_tar
}

#[derive(Debug, PartialEq, Eq)]
pub enum PkgType {
    Debian,
    Unknown,
}

impl Display for PkgType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Debian => write!(f, "DebainPackage"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

pub fn get_pkgtype<P: AsRef<Path>>(dir: P) -> PkgType {
    if is_debian(&dir) {
        PkgType::Debian
    } else {
        PkgType::Unknown
    }
}

pub fn convert() -> Result<(), std::io::Error> {
    match get_pkgtype("./") {
        PkgType::Debian => debian(),
        PkgType::Unknown => Err(std::io::Error::from(
            std::io::ErrorKind::NotFound,
        )),
    }
}

fn load_debinfo() -> Result<PackageData, std::io::Error> {
    use crate::modules::repo::types::apt;
    let current_dir = std::env::current_dir()?;
    let control_file = current_dir.join("control/control");
    let control_file = fs::read_to_string(&control_file)?;
    let control_data = apt::parse_control_file(&control_file)
        .map_err(|e| std::io::Error::other(e))?;
    apt::to_package_data(control_data).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, e)
    })
}

fn debian() -> Result<(), std::io::Error> {
    let current_dir = std::env::current_dir()?;

    // 展開先ディレクトリの作成
    let data_dir = current_dir.join("data");
    let control_dir = current_dir.join("control");
    fs::create_dir_all(&data_dir)?;
    fs::create_dir_all(&control_dir)?;

    // data.tar.* と control.tar.* のファイルを探す
    let entries = fs::read_dir(&current_dir)?;
    for entry in entries {
        let entry = entry?;
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();

        if file_name.starts_with("data.tar.") {
            extract_archive(entry.path(), data_dir.clone())?;
            fs::remove_file(entry.path())?;
        } else if file_name.starts_with("control.tar.") {
            extract_archive(entry.path(), control_dir.clone())?;
            fs::remove_file(entry.path())?;
        }
    }
    // この時点で、dataディレクトリと、コントロールディレクトリのみとなっている。
    // コントロールディレクトリにあるはずのcontrolファイルから、PackageDataを取得する
    let package_data = load_debinfo()?;
    dprintln!("{}", package_data);
    fs::create_dir_all("ipak/scripts")?;
    fs::write(
        "ipak/project.yaml",
        serde_yaml::to_string(&package_data).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e,
            )
        })?,
    )?;
    templates::set(PkgType::Debian)?;
    Ok(())
}
