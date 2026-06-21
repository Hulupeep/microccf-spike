# Endpoint Catalog

This catalog records only the endpoints used by the microCCF spike. It is not a
complete Cognitum Seed API reference.

## microCCF Service

Base URL on the attached Seed:

```text
http://169.254.42.1:8081
```

When the Seed is on a reachable LAN, the same service should be available at:

```text
http://<seed-lan-ip>:8081
```

### `GET /health`

No auth.

Example response:

```json
{
  "service": "microccf",
  "status": "ok",
  "warning": "Spike build, NOT v1.0"
}
```

### `GET /state`

No auth.

Returns the fake counter state plus last observed Seed status fields.

Example response:

```json
{
  "service": "microccf",
  "warning": "Spike build, NOT v1.0",
  "tick_count": 37,
  "total_seen": 37,
  "counts": {"ctx_a": 24, "ctx_b": 13},
  "error_count": 0,
  "witness_chain_length": 5712,
  "total_vectors": 2204,
  "epoch": 5246,
  "paired": true,
  "last_ingest_id": "9000000000037"
}
```

### `GET /metrics`

No auth.

Returns the same counters plus recent latency samples for:

- `status`
- `query`
- `ingest`

The `ingest` sample includes both `POST /store/ingest` and
`PUT /store/vectors/{id}/metadata`.

### `GET /version`

No auth.

Example response:

```text
microccf 0.1.0 - Spike build, NOT v1.0
```

## Cognitum Seed Endpoints Used

Base URL from inside the Seed:

```text
https://127.0.0.1:8443
```

The service currently accepts the local certificate with
`allow_invalid_certs = true`.

### `GET /api/v1/status`

No bearer token required in this run.

Used each tick to capture:

- `witness_chain_length`
- `total_vectors`
- `epoch`
- `paired`

### `POST /api/v1/store/query`

No bearer token required in this run.

Request body:

```json
{
  "vector": [0.13, 0.125, 0.25, 0.375, 0.5, 0.625, 0.75, 0.875],
  "k": 1,
  "limit": 1
}
```

Used to prove the Seed query path remains available while microCCF is writing.

### `POST /api/v1/store/ingest`

Bearer token required.

Request body:

```json
{
  "vectors": [
    [
      9000000000013,
      [0.13, 0.125, 0.25, 0.375, 0.5, 0.625, 0.75, 0.875]
    ]
  ]
}
```

Observed response shape:

```json
{
  "accepted": true,
  "count": 1,
  "new_epoch": 5198,
  "rejected": 0,
  "verified": false,
  "witness_head": "..."
}
```

Discovery: this firmware accepts a top-level `metadata` field on ingest but does
not attach it to the vector. Do not rely on direct ingest metadata for Gate C.

### `PUT /api/v1/store/vectors/{id}/metadata`

Bearer token required.

Request body:

```json
{
  "metadata": [
    {"field_id": 0, "value": {"String": "microccf"}},
    {"field_id": 1, "value": {"String": "witness-test"}},
    {"field_id": 2, "value": {"String": "ctx_a"}},
    {"field_id": 3, "value": {"U64": 13}},
    {"field_id": 4, "value": {"U64": 13}}
  ]
}
```

Observed response shape:

```json
{
  "id": 9000000000013,
  "new_epoch": 5198,
  "updated": true,
  "witness_head": "..."
}
```

This is the enforceable metadata path used by the spike.

### `GET /api/v1/store/vectors/{id}`

No bearer token required in this run.

Used to prove that metadata persisted:

```json
{
  "id": 9000000000013,
  "vector": [0.13, 0.125, 0.25, 0.375, 0.5, 0.625, 0.75, 0.875],
  "metadata": [
    {"field_id": 0, "value": {"String": "microccf"}},
    {"field_id": 1, "value": {"String": "witness-test"}},
    {"field_id": 2, "value": {"String": "ctx_a"}},
    {"field_id": 3, "value": {"U64": 13}},
    {"field_id": 4, "value": {"U64": 13}}
  ],
  "deleted": false
}
```

## Known Gaps

- WiFi/LAN access was not proven. The Seed only exposed setup/AP WiFi
  `192.168.4.1/24` during this run.
- Browser dashboard visibility was not proven. Store/query visibility by ID was
  proven.
- This catalog should be re-run after any Cognitum firmware update because the
  direct-ingest metadata behavior may change.
