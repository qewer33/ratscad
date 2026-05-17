#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RATTY_DIR="${REPO_ROOT}/references/ratty"
RATTY_BIN="${RATTY_DIR}/target/release/ratty"
RATTY_REPO="https://github.com/orhun/ratty.git"
RATSCAD_BIN="${REPO_ROOT}/target/debug/ratscad"

if [[ ! -d "${RATTY_DIR}" ]]; then
  echo ">> Cloning Ratty into ${RATTY_DIR}"
  mkdir -p "${REPO_ROOT}/references"
  git clone "${RATTY_REPO}" "${RATTY_DIR}"
fi

echo ">> Building ratscad (debug)"
(cd "${REPO_ROOT}" && cargo build)

echo ">> Building Ratty (release — first build is slow, subsequent rebuilds are incremental)"
(cd "${RATTY_DIR}" && cargo build --release)

echo ">> Launching Ratty hosting ratscad"
exec "${RATTY_BIN}" -e "${RATSCAD_BIN}" "$@"
