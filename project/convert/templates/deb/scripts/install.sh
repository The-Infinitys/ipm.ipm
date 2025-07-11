#!/bin/sh
set -e

# グローバルモード以外は実行しない
if [ "$IPAK_INSTALL_MODE" != "global" ]; then
    echo "Error: Only global installation mode is supported"
    exit 1
fi

# インストール先のルートディレクトリ
DATA_DIR="/"

# dataディレクトリから必要なファイルをコピー
if [ -d "data/usr" ]; then
    cp -r data/usr/* "$DATA_DIR/usr/"
fi

if [ -d "data/etc" ]; then
    cp -r data/etc/* "$DATA_DIR/etc/"
fi

if [ -d "data/opt" ]; then
    cp -r data/opt/* "$DATA_DIR/opt/"
fi

# その他のディレクトリも必要に応じてコピー
for dir in data/*/; do
    base_dir=$(basename "$dir")
    case "$base_dir" in
        "usr"|"etc"|"opt"|".")
            continue
            ;;
        *)
            if [ -d "data/$base_dir" ]; then
                cp -r "data/$base_dir" "$DATA_DIR/"
            fi
            ;;
    esac
done

# post-install スクリプトが存在する場合は実行
if [ -f "control/postinst" ]; then
    sh ./control/postinst
fi

echo "Installation of $IPAK_PROJECT_NAME $IPAK_PROJECT_VERSION completed successfully."
