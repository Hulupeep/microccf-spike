#!/usr/bin/env bash
set -euo pipefail

SEED_HOST="${MICROCCF_SEED_HOST:-ccf-seed}"
TARGET="${MICROCCF_TARGET:-arm-unknown-linux-gnueabihf}"
BIN="target/${TARGET}/release/microccf"

if [ ! -x "$BIN" ]; then
  echo "missing binary: $BIN" >&2
  exit 1
fi

if [ -z "${MICROCCF_SEED_TOKEN:-}" ]; then
  echo "MICROCCF_SEED_TOKEN is required and must not be committed" >&2
  exit 1
fi

tmpdir="$(ssh "$SEED_HOST" 'mktemp -d')"
token_file="$(mktemp)"
trap 'rm -f "$token_file"' EXIT
chmod 600 "$token_file"
printf '%s\n' "$MICROCCF_SEED_TOKEN" >"$token_file"

scp "$BIN" "$SEED_HOST:${tmpdir}/microccf"
scp deploy/microccf.service "$SEED_HOST:${tmpdir}/microccf.service"
scp deploy/config.example.toml "$SEED_HOST:${tmpdir}/config.toml"
scp "$token_file" "$SEED_HOST:${tmpdir}/token"

ssh "$SEED_HOST" "TMPDIR='${tmpdir}' bash -s" <<'REMOTE_EOF'
set -euo pipefail

sudo install -d -m 755 /opt/microccf /etc/microccf
if ! id microccf >/dev/null 2>&1; then
  sudo useradd --system --home /nonexistent --shell /usr/sbin/nologin microccf
fi

sudo install -m 755 "$TMPDIR/microccf" /opt/microccf/microccf
sudo install -m 644 "$TMPDIR/config.toml" /etc/microccf/config.toml
sudo install -m 600 -o microccf -g microccf "$TMPDIR/token" /etc/microccf/token
sudo install -m 644 "$TMPDIR/microccf.service" /etc/systemd/system/microccf.service
sudo systemctl daemon-reload
sudo systemctl enable microccf >/dev/null
sudo systemctl restart microccf
systemctl is-active microccf
REMOTE_EOF
