# Third-party components

`vigil` is licensed under Apache-2.0 (see `LICENSE`). It links the open-source components listed
below, each governed by its own license; those terms prevail for those components. Required
attributions are in `NOTICE`.

This file is generated from the resolved dependency graph — do not edit by hand.

## Direct dependencies

| Component | Used by | Why it is here                                                                        |
|---|---|---------------------------------------------------------------------------------------|
| `serde`, `serde_json` | `vigil-model`, `vigil-rules`, `vigil-report` | The wire contract is JSON; hand-rolling it would be a worse dependency than this one. |
| `serde_yaml` | `vigild` | The configuration file an operator edits.                                             |
| `ratatui` (+ `crossterm`) | `vigil` | The console. Deliberately absent from the daemon.                                     |
| `clap` | `vigil`, `vigild` | The command line of both binaries.                                                    |
| `sha2` | `vigild`, `vigil-collect` | SHA-256 and for SSH key fingerprints.                                                 |
| `rustix` | `vigil-collect` | `statvfs` on Linux: how full a filesystem is lives in no file under `/proc`.          |
