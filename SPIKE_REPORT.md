# microCCF Spike Report

**Date:** 2026-06-21
**Scope:** public throwaway Cognitum Seed deployment spike for CCF architecture
validation.

## Executive Verdict

The attached Seed can run a third-party Rust service beside `cognitum-agent`,
serve an external state endpoint, call the local Seed API, and write
witness-backed vectors with persisted `source=microccf` metadata.

This supports continuing with a Seed-hosted CCF architecture, but not by
promoting this repo into product code. This repo is fake math and infrastructure
only.

## Question-by-question Findings

### Q1 - Can a custom Rust binary run on the Cognitum Seed?

**Verdict:** yes.

Evidence:

- The attached Seed is `armv7l` on 32-bit Raspbian 13, not aarch64.
- Host cross-build succeeded for `arm-unknown-linux-gnueabihf`.
- Release binary size: about 6.4 MB.
- Service installed at `/opt/microccf/microccf`.

Implication: the original aarch64 assumption is stale for this Seed. CCF should
either cross-build `arm-unknown-linux-gnueabihf` or build on-device.

### Q2 - What's the bearer-token plumbing in practice?

**Verdict:** works, with care.

The deploy script requires `MICROCCF_SEED_TOKEN` in the local environment and
copies it to `/etc/microccf/token` on the Seed with mode `0600`, owner
`microccf:microccf`. The token is not committed, not echoed, and not interpolated
into the SSH command line.

### Q3 - Can a third-party Rust service co-exist with `cognitum-agent`?

**Verdict:** yes.

`microccf.service` runs as a separate `microccf` system user, listens on
`0.0.0.0:8081`, and talks to `cognitum-agent` over `https://127.0.0.1:8443`.

### Q4 - What's the realistic latency of Seed API calls from on-device?

**Verdict:** low enough for the architecture gate.

Short benchmark from the patched build:

| API call | Count | Mean | p50 | p95 | p99 |
|---|---:|---:|---:|---:|---:|
| `GET /api/v1/status` | 37 | 2.84 ms | 1.20 ms | 1.52 ms | 59.29 ms |
| `POST /api/v1/store/query` | 37 | 1.31 ms | 1.24 ms | 1.59 ms | 1.66 ms |
| ingest + metadata update | 37 | 23.44 ms | 21.78 ms | 30.91 ms | 32.51 ms |

The status p99 includes one outlier. The write path includes both vector ingest
and typed metadata update.

### Q5 - Can microCCF write to the witness chain?

**Verdict:** yes.

The service writes one vector per tick. Each vector ID is deterministic:

```text
9000000000000 + tick
```

Then it persists typed metadata:

| Field | Value |
|---:|---|
| 0 | `microccf` |
| 1 | `witness-test` |
| 2 | context bucket |
| 3 | tick |
| 4 | count |

Verified example:

```json
{
  "id": 9000000000013,
  "metadata": [
    {"field_id": 0, "value": {"String": "microccf"}},
    {"field_id": 1, "value": {"String": "witness-test"}},
    {"field_id": 2, "value": {"String": "ctx_a"}},
    {"field_id": 3, "value": {"U64": 13}},
    {"field_id": 4, "value": {"U64": 13}}
  ]
}
```

Important API discovery: `POST /api/v1/store/ingest` accepts an optional
`metadata` field but does not attach it to the vector in this firmware. The
enforceable path is:

1. `POST /api/v1/store/ingest` with `{"vectors":[[id, vector]]}`.
2. `PUT /api/v1/store/vectors/{id}/metadata` with typed metadata.

### Q6 - What does reboot survival look like?

**Verdict:** service survives reboot.

`microccf.service` is enabled under systemd and came back after Seed reboot. The
host USB link needed manual NetworkManager profile rebinding because the gadget
interface name changed. After the host link was restored, the service already had
224 seconds uptime, 225 ticks, and zero errors.

Architectural implication: product Gate C needs a stable Seed runner/network
profile, not just a working systemd unit.

### Q7 - What does an external query look like over WiFi?

**Verdict:** not proven in this run.

The Seed exposed:

- `usb0`: `169.254.42.1/16`
- `wlan0`: `192.168.4.1/24`

`192.168.4.1` is the setup/AP-side address and was not reachable from the host's
LAN path. USB-host queries to `http://169.254.42.1:8081` are proven. LAN WiFi
queries need the Seed joined to the same reachable WiFi network as the operator
machine.

### Q8 - What does the Cognitum dashboard show about microCCF?

**Verdict:** store/query visibility is proven; browser dashboard visibility is
not yet proven.

The Seed store can look up the microCCF vector by ID and returns the persisted
metadata. The browser dashboard check should be repeated once the Seed is on a
reachable WiFi network or with a browser connected to the Seed setup AP.

## Resource Baseline

Sample process snapshot:

```text
USER       COMMAND    %CPU  %MEM   RSS
microccf   microccf    0.5   1.2  5880 KB
```

The service uses a separate system user and no write paths outside:

- `/opt/microccf/`
- `/etc/microccf/`
- `/etc/systemd/system/microccf.service`

## Runtime State Example

```json
{
  "service": "microccf",
  "warning": "Spike build, NOT v1.0",
  "tick_count": 37,
  "total_seen": 37,
  "error_count": 0,
  "witness_chain_length": 5712,
  "total_vectors": 2204,
  "epoch": 5246,
  "paired": true,
  "last_ingest_id": "9000000000037"
}
```

## Architectural Risks Discovered

- The attached Seed is 32-bit ARM, not aarch64.
- Direct ingest metadata is ignored; metadata must be persisted through the
  vector metadata endpoint.
- USB gadget interface names can change after reboot, so a branch-protected Gate
  C runner must own link recovery.
- WiFi LAN access was not proven because the Seed was only exposing setup/AP
  WiFi in this run.

## Recommendation

Proceed with the Seed-hosted architecture track, but treat these as required
follow-up tickets before canonical CCF Gate C:

1. Build a stable Seed runner/profile that keeps `169.254.42.2/16` bound after
   reboots and cable changes.
2. Decide whether canonical CCF uses direct REST calls, MCP tools, or a local
   Seed-side adapter for vector writes.
3. Include metadata-write verification in Gate C; do not accept witness writes
   that cannot be queried back by ID with `source=ccf` or equivalent metadata.
4. Run the WiFi/LAN external query once the Seed is joined to a reachable network.

## What Does Not Carry Over To v1.0

- Fake four-context taxonomy.
- HashMap counters.
- The `microccf` HTTP API as a product API.
- Hard-coded witness-test vector IDs.
- Any CCF math or policy semantics.

## What Carries Over To v1.0

- A Rust service can run on the Seed as a separate systemd unit.
- The service can call the local Seed API with bearer-token auth.
- The service can write vectors that advance the witness chain.
- The service can persist and query back typed metadata.
- Seed-hosted state can be served externally over HTTP.
