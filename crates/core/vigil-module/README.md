# vigil-module

What a subject watched by this agent tells the two binaries about itself.

## The decision it follows from

One subject is one crate — reading, parsers, rules and screen together
(`docs/designs/2026-09-12-DESIGN-modules.md`). That only pays if the binaries stop keeping
lists of what exists: a table of collector names, a table pairing a collector with its rules,
a numbered list of screens, a map from a finding to the row it is about. Every such table is
a place a new module is forgotten, and forgotten quietly.

So the module answers, and the root only asks. `bin/vigild` holds one `Vec<Box<dyn Module>>`
and nothing else about any subject; `bin/vigil` walks the same list for the sections to draw.

## What a module answers

- **`name`** — the name of its reading, the word in `collectors:` in the configuration file
  and the word an operator types into `vigild collector enable`;
- **`subject`, `every_seconds`, `unit`** — what it watches in one sentence, how often, and the
  systemd unit (if any) that has to be running for it to read anything;
- **`settings_key`** and `Settings` — its own fragment of `vigil.yaml`, already parsed into a
  value, plus the clock it stamps a reading with. A module never reads the file and never
  reads the host's time;
- **`collector`, `rules`** — the two halves of the watch: what it reads, and what it makes of
  the difference between two readings;
- **`families`, `row_of`** — which finding keys are its own, and which row of its reading a
  finding is about. The default is the whole key after the family, which is what a collector
  that keys its rows the ordinary way wants;
- **`section`** — its screen, described with `vigil-view` and drawn by nobody here.

## Dependencies

`vigil-model`, `vigil-collect` (the `Collector` port), `vigil-rules` (`RuleSet`), `vigil-view`
(`Section`), `serde`/`serde_json` for the settings fragment. No io of its own.

## Context

```
vigil-collect ─┐
vigil-rules  ──┼─► vigil-module ◄── collectors/vigil-<subject>/modules/
vigil-view   ──┘                              │
                                              ├──► bin/vigild  (collector + rules)
                                              └──► bin/vigil   (section + row_of)
```
