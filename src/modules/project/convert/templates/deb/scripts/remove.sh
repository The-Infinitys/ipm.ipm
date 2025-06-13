#!/bin/sh
set -e

# グローバルモード以外は実行しない
if [ "$IPAK_REMOVE_MODE" != "global" ]; then
    echo "Error: Only global removal mode is supported"
    exit 1
fi

# 削除対象のルートディレクトリ
DATA_DIR="/"

# prerm スクリプトが存在する場合は実行
if [ -f "control/prerm" ]; then
    sh ./control/prerm
fi

# dataディレクトリのファイル一覧を使って削除
find data -type f | while read -r file; do
    # dataディレクトリからの相対パスを取得
    rel_path="${file#data/}"
    # インストール先のファイルを削除
    target_file="$DATA_DIR/$rel_path"
    if [ -f "$target_file" ]; then
        rm -f "$target_file"
    fi
done

# 空になったディレクトリを削除
find data -type d | sort -r | while read -r dir; do
    # dataディレクトリからの相対パスを取得
    rel_path="${dir#data/}"
    # インストール先のディレクトリを確認
    target_dir="$DATA_DIR/$rel_path"
    if [ -d "$target_dir" ]; then
        # ディレクトリが空の場合のみ削除
        rmdir "$target_dir" 2>/dev/null || true
    fi
done

# postrm スクリプトが存在する場合は実行
if [ -f "control/postrm" ]; then
    sh ./control/postrm
fi

echo "Removal of $IPAK_PROJECT_NAME $IPAK_PROJECT_VERSION completed successfully."
