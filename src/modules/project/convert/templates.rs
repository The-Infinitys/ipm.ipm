use super::PkgType;
use ipak::utils::files::file_creation;
struct SetUpItem {
    path: String,
    content: String,
}

/// 指定されたファイルリストに基づいてファイルを生成します。
///
/// 各ファイルは、そのパスとコンテンツに従って作成されます。
/// ファイル作成中にエラーが発生した場合、具体的なエラーメッセージと共に
/// `std::io::Error` が返されます。
///
/// # 引数
///
/// * `setup_list` - 生成するファイルのパスとコンテンツのリスト。
///
/// # 戻り値
///
/// ファイル生成がすべて成功した場合は `Ok(())`、一つでも失敗した場合は `std::io::Error` を返します。
fn setup_files(
    setup_list: Vec<SetUpItem>,
) -> Result<(), std::io::Error> {
    for item in setup_list {
        // file_creation の結果を直接伝播させ、エラー発生時に詳細な情報を付与する
        file_creation(&item.path, &item.content).map_err(
            |e| {
                std::io::Error::new(
                    e.kind(),
                    format!(
                        "Failed to create file '{}': {}",
                        item.path, e
                    ),
                )
            },
        )?;
    }
    Ok(())
}

pub fn set(pkg_type: PkgType) -> Result<(), std::io::Error> {
    let setup_list = match pkg_type {
        PkgType::Debian => {
            vec![
                SetUpItem {
                    path: "ipak/scripts/install.sh".to_string(),
                    content: include_str!(
                        "templates/deb/scripts/install.sh"
                    )
                    .to_string(),
                },
                SetUpItem {
                    path: "ipak/scripts/remove.sh".to_string(),
                    content: include_str!(
                        "templates/deb/scripts/remove.sh"
                    )
                    .to_string(),
                },
                SetUpItem {
                    path: "ipak/scripts/purge.sh".to_string(),
                    content: include_str!(
                        "templates/deb/scripts/purge.sh"
                    )
                    .to_string(),
                },
            ]
        }
        PkgType::Unknown => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Unknown package type",
            ));
        }
    };
    setup_files(setup_list)?;
    Ok(())
}
