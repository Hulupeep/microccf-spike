#!/usr/bin/env bash
set -euo pipefail

SEED_HOST="${MICROCCF_SEED_HOST:-ccf-seed}"

ssh "$SEED_HOST" <<'REMOTE_EOF'
set -euo pipefail
sudo systemctl disable --now microccf >/dev/null 2>&1 || true
sudo rm -f /etc/systemd/system/microccf.service
sudo systemctl daemon-reload
sudo rm -rf /opt/microccf /etc/microccf
REMOTE_EOF

