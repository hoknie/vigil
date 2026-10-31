# vigil-store

> What the agent keeps on its own disk.

## Why

The store is plain files: an append-only NDJSON journal of findings and one JSON snapshot per
collector. A restart picks up where the agent left off, and history at an incident is read with
`cat` and `jq`. Every write is atomic, durable and `0600`. A damaged file costs only what it held,
and the daemon starts.

The `Store` port and its conformance suite describe what any backend must do. The files backend,
behind the `files` feature, is the one in use.

## What's inside

| File under the state directory | What it holds |
|---|---|
| `findings/journal.ndjson` | the journal of findings, one per line |
| `baselines/<collector>.json` | the last reading of each collector |
| `outgoing/<receiver>.ndjson` | findings a receiver has yet to take, oldest first, under a ceiling that displaces the oldest |
| `outgoing/<receiver>.batch` | the batch a receiver refused: its `batch_id`, its size and the first and last `event_id` |

A refused batch is sent again exactly as it was, under the same id, in this run or after a
restart, so a receiver can tell a retry from a new batch. The `.batch` file is removed once the
batch is taken or can no longer be retried.

`create_owner_only` and `create_owner_only_directory` are exported for the `ndjson` receiver's own
file.

## Depends on

- `vigil-model`
- `serde_json`

## Where it fits

```text
vigil-model ──► vigil-store ──► vigild · the watch round, delivery, the ndjson receiver
```
