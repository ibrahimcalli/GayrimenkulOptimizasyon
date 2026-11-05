#!/usr/bin/env bash
# Gayrimenkul360 - İstemci (danışman programı) başlatıcı (Linux/macOS)
cd "$(dirname "$0")" || exit 1

if [ -x "target/release/gayrimenkul-client" ]; then
  BIN="target/release/gayrimenkul-client"
elif [ -x "target/debug/gayrimenkul-client" ]; then
  BIN="target/debug/gayrimenkul-client"
else
  echo "İstemci ikili dosyası bulunamadı."
  echo "Önce derleyin:  ./build-release.sh"
  exit 1
fi

exec "$BIN"
