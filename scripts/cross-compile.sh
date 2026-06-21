#!/usr/bin/env bash
set -euo pipefail

TARGET="${MICROCCF_TARGET:-arm-unknown-linux-gnueabihf}"
start="$(date +%s)"
cargo build --target "$TARGET" --release
end="$(date +%s)"
bin="target/${TARGET}/release/microccf"
ls -lh "$bin"
printf 'target=%s build_seconds=%s\n' "$TARGET" "$((end - start))"

