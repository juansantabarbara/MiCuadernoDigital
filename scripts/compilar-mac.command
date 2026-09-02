#!/bin/zsh
set -e
cd "$(dirname "$0")/.."
echo "== MiCuadernoDigital · compilación macOS =="
if ! command -v xcode-select >/dev/null || ! xcode-select -p >/dev/null 2>&1; then
  echo "Instalando herramientas de línea de comandos de Xcode..."
  xcode-select --install || true
  echo "Cuando termine la instalación, vuelve a ejecutar este archivo."
  exit 1
fi
if ! command -v rustup >/dev/null 2>&1; then
  echo "Falta Rust. Instálalo desde https://rustup.rs y vuelve a ejecutar este archivo."
  exit 1
fi
if ! command -v npm >/dev/null 2>&1; then
  echo "Falta Node.js/npm."
  exit 1
fi
npm install
npm run build
echo "Listo. Busca MiCuadernoDigital.app y el .dmg en src-tauri/target/release/bundle/"
