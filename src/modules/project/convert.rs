use ipak::utils::archive::extract_archive;
use std::fs;
use std::path::Path;

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
        } else if file_name.starts_with("control.tar.") {
            extract_archive(entry.path(), control_dir.clone())?;
        }
    }

    Ok(())
}
