#!/usr/bin/env bash
set -euo pipefail

SEED_HOST="${MICROCCF_SEED_HOST:-ccf-seed}"

ssh "$SEED_HOST" 'curl --fail --silent http://127.0.0.1:8081/metrics'
printf '\n'

