#!/bin/sh

set -eu

RUST_TOOLCHAIN_VERSION="1.97.1"
WASM_TARGET="wasm32-unknown-unknown"
WORKER_BUILD_VERSION="0.8.5"

worker_cargo_home="${CARGO_HOME:-${HOME:?HOME must be set}/.cargo}"
PATH="${worker_cargo_home}/bin:${PATH}"
export PATH

if ! command -v rustup >/dev/null 2>&1; then
  if ! command -v curl >/dev/null 2>&1; then
    printf '%s\n' "curl is required to install the Rust toolchain." >&2
    exit 1
  fi

  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs |
    sh -s -- -y --profile minimal --default-toolchain none
fi

if ! rustup run "${RUST_TOOLCHAIN_VERSION}" rustc --version >/dev/null 2>&1; then
  rustup toolchain install "${RUST_TOOLCHAIN_VERSION}" --profile minimal
fi

if ! rustup target list --installed --toolchain "${RUST_TOOLCHAIN_VERSION}" |
  grep -Fqx "${WASM_TARGET}"; then
  rustup target add "${WASM_TARGET}" --toolchain "${RUST_TOOLCHAIN_VERSION}"
fi

installed_worker_build_version="$(worker-build --version 2>/dev/null || true)"
case "${installed_worker_build_version}" in
  "${WORKER_BUILD_VERSION}" | "worker-build ${WORKER_BUILD_VERSION}") ;;
  *)
    rustup run "${RUST_TOOLCHAIN_VERSION}" cargo install worker-build \
      --version "${WORKER_BUILD_VERSION}" \
      --locked
    ;;
esac

exec worker-build --release
