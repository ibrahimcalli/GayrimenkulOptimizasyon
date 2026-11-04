#!/usr/bin/env bash
# Tüm bileşenleri yayın (release) modunda derler.
set -e
cd "$(dirname "$0")"
echo "=== Gayrimenkul360 derleniyor (release) ==="
cargo build --release
echo ""
echo "Derleme tamamlandı. Oluşan dosyalar:"
echo "  target/release/gayrimenkul-server"
echo "  target/release/gayrimenkul-client"
