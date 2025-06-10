use std::path::Path;

pub fn convert() -> Result<(), std::io::Error> {
    let files = ["control.tar.gz", "control.tar.xz", "control.tar.bz2", "data.tar.gz", "data.tar.xz", "data.tar.bz2", "debian-binary"];
    for file in files.iter() {
        if !Path::new(file).exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("{} not found in current directory", file),
            ));
        }
    }
    Ok(())
}
