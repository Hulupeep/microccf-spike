#!/usr/bin/env bash
set -euo pipefail

scripts/cross-compile.sh
deploy/install.sh

SEED_HOST="${MICROCCF_SEED_HOST:-ccf-seed}"
ssh "$SEED_HOST" 'systemctl status microccf --no-pager | sed -n "1,40p"'
curl --fail --silent --show-error http://169.254.42.1:8081/health
printf '\n'
curl --fail --silent --show-error http://169.254.42.1:8081/state
printf '\n'

