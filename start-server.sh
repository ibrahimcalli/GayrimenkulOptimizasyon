#!/usr/bin/env bash
# Gayrimenkul360 - Ofis Sunucusu başlatıcı (Linux/macOS)
cd "$(dirname "$0")" || exit 1

if [ -x "target/release/gayrimenkul-server" ]; then
  BIN="target/release/gayrimenkul-server"
elif [ -x "target/debug/gayrimenkul-server" ]; then
  BIN="target/debug/gayrimenkul-server"
else
  echo "Sunucu ikili dosyası bulunamadı."
  echo "Önce derleyin:  ./build-release.sh"
  exit 1
fi

echo "Sunucu başlatılıyor... (Bu pencereyi açık tutun)"
exec "$BIN"
