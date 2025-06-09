use super::list;
use cmd_arg::cmd_arg;
pub fn pkg(
    args: Vec<&cmd_arg::Option>,
) -> Result<(), std::io::Error> {
    if args.is_empty() {
        return Err(std::io::Error::from(
            std::io::ErrorKind::NotFound,
        ));
    }
    let sub_cmd = args.first().unwrap().opt_str.to_string();
    let sub_args = args[1..].to_vec();
    match sub_cmd.as_str() {
        "search" => {
            let pacakges = sub_args
                .iter()
                .map(|arg| -> String { arg.opt_str.to_string() })
                .collect();
            search_pkgs(pacakges)
        }
        _ => Err(std::io::Error::from(
            std::io::ErrorKind::NotFound,
        )),
    }
}
fn search_pkgs(
    packages_name: Vec<String>,
) -> Result<(), std::io::Error> {
    let packages = list::packages()?;
    for name in packages_name {
        if let Some(pkg) = packages
            .iter()
            .find(|p| p.info.about.package.name == name)
        {
            println!("{}", pkg.info);
            // ここでパッケージの取得処理を追加できます
        }
    }
    Ok(())
}
