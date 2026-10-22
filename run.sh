#!/usr/bin/env bash
# Roda o projeto sem decorar comando. Uso: ./run.sh [dev|cli|mcp|test|build|lint|help]
# Sem argumento: dev. Stack: rust + tauri.
set -euo pipefail
cd "$(dirname "$0")"

usage() {
  cat <<'EOF2'
uso: ./run.sh [comando] (padrão: dev)
  dev    abre o app (Tauri) em modo de desenvolvimento
  cli    compila só o CLI e o servidor MCP, sem Tauri (target/release/salamandra e salamandra-mcp)
  mcp    roda o servidor MCP por stdio (para testar à mão)
  test   testes do CLI, do MCP e do núcleo (sem Tauri); use "test --workspace" para tudo
  build  build de release do app (instaladores)
  lint   clippy no workspace inteiro
EOF2
}

case "${1:-dev}" in
  dev) shift || true; (cd app && npm install --silent && npm run tauri dev -- "$@") ;;
  cli) shift || true; cargo build --release -p salamandra-cli -p salamandra-mcp "$@"
       echo "pronto: target/release/salamandra e target/release/salamandra-mcp" ;;
  mcp) shift || true; cargo run -q -p salamandra-mcp "$@" ;;
  test) shift || true; cargo test "$@" ;;
  build) shift || true; (cd app && npm install --silent && npm run tauri build -- "$@") ;;
  lint) shift || true; cargo clippy --workspace --all-targets -- -D warnings "$@" ;;
  help|-h|--help) usage ;;
  *) usage >&2; exit 1 ;;
esac
