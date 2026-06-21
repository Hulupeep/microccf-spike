# microCCF Spike

Throwaway Cognitum Seed deployment spike. Fake math. Real infrastructure.

This repository is not `ccf-core`, not v1.0, and not a production component.
It exists to answer one architecture question: can a third-party Rust service
run on the Cognitum Seed, write witness-backed test events, survive reboot, and
serve externally queryable state?

The math is intentionally fake:

```text
context = bucket(vector[0])
count[context] += 1
```

Do not add QAC, kappa, Sinkhorn, min-cut, gates, coherence, or any canonical
CCF logic here.

## Current Seed Target

The attached Seed reports `armv7l` on 32-bit Raspbian 13, so this spike uses:

```bash
cargo build --target arm-unknown-linux-gnueabihf --release
```

The original aarch64 assumption is recorded as stale for this device.

## Local Build

```bash
cargo test
cargo build
```

## Cross Build

```bash
scripts/cross-compile.sh
```

## Deploy

Set a token in the environment without committing it:

```bash
export MICROCCF_SEED_TOKEN='seed_...'
export MICROCCF_SEED_HOST=ccf-seed
scripts/deploy-and-run.sh
```

The install writes only:

- `/opt/microccf/`
- `/etc/microccf/`
- `/etc/systemd/system/microccf.service`

The service listens on `0.0.0.0:8081`. The default loop writes one witness-test
vector every 30 seconds; reduce `poll_interval_ms` only for short benchmark runs.
