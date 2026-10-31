# vigil-module

> What a watched subject tells the two programs about itself.

## Why

One subject is one crate: its reading, parsers, rules and screen together. Each module answers for
itself, and the programs ask. `vigild` holds one `Vec<Box<dyn Module>>`, and `vigil` walks the same
list for the sections it draws. A new subject joins by being added to that list.

## What's inside

| | |
|---|---|
| **`name`** | the name of its reading — the word in `collectors:` and in `vigild collector enable` |
| **`subject`, `every_seconds`, `unit`** | what it watches, how often, and the systemd unit it needs |
| **`settings_key`, `Settings`** | its own block of the configuration, already parsed, plus the clock it stamps a reading with |
| **`follows_the_file`** | whether an edit to its block is taken up while the daemon runs; today that is `files` |
| **`collector`, `rules`** | what it reads and what it makes of the difference |
| **`families`, `row_of`** | which finding keys belong to it, and which row a finding is about |
| **`section`** | its screen, described with `vigil-view` |

## Depends on

- `vigil-model` · `vigil-collect` · `vigil-rules` · `vigil-view`
- `serde`, `serde_json` for the settings block

## Where it fits

```text
vigil-collect ─┐
vigil-rules  ──┼──► vigil-module ◄── collectors/vigil-<subject>/modules
vigil-view   ──┘                              │
                                              ├──► vigild · collector and rules
                                              └──► vigil  · section and row_of
```
