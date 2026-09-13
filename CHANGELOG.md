# Changelog

All notable changes by release tag, newest first.
Format:
- [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
- [SemVer](https://semver.org/spec/v2.0.0.html)

## [1.0.6] - 09/--/2026

### Changed
- `vigil.yaml` is read in two passes: the daemon's own keys, then the key each module names as its own. The daemon's configuration struct holds no module's fields any more — it carries each block whole and hands it over — and a module validates its own block at start-up rather than at the first reading. A key nobody knows is still refused, at whichever of the two levels it was written
- the switch for the arguments a program was launched with moved into the block of the module that reads them: `launches:` / `record_arguments:`, where it was the daemon's own `record_launch_arguments`
- the console keeps no table of sections and no map from a finding to the screen that holds its object: the numbers 1 to 9, every title, every sentence on the main screen and every walk from a finding to its row are read from the modules this build links. Adding a module is a line in a `Cargo.toml` and a line in each binary's list, and the two lists are compared with each other and with the crates on disk
- the hint line at the foot of the screen offers the keys the list under the cursor actually answers to: `s` where that list can be put in an order, `t` where it has a second view, `o` only on a finding — where before it was a table of screen names that had gone stale twice
- asking the modules rather than a table costs 1.5 µs a call and about 21 µs of a main-screen frame that takes 190 µs; the console draws a frame on a key press and every couple of seconds
- every subject this agent watches is a crate of its own: `resources` and `files` are the last two, and they share the section the console calls `system`. The daemon's table of collector names and its hand-written pairs of collector and rule set are gone — it builds the watch cycle from the module list alone — and the console holds no screen, no list state and no detail for any subject
- what the two of them watch is a setting each module reads for itself: the free-space limits and the list of paths reach the module as its own slice of `vigil.yaml`, and the defaults are the module's rather than the daemon's
- drawing those two lists costs what it did: on musl, 0.4 µs to list the rows and 4.1 µs to build the cells of a window of forty for the host, 0.4 µs and 2.1 µs for the watched paths; the readings themselves are unchanged at 0.07 ms and 0.02 ms apiece
- `processes` and `launches` are crates of their own, and the section they share is assembled from both: two modules declare the same screen, the console concatenates their lists, and a finding walks to the row in whichever of the two readings holds it
- the audit spool and its plugin belong to the module that writes and reads them: `bin/vigil-audit-plugin` depends on `vigil-launches` and no longer on the core reading crate
- what a module keeps of a command line is now a setting the module reads for itself (`launches.record_arguments`), handed over by the daemon rather than read from a field of the daemon's own configuration struct
- the crate that watches listening sockets is `vigil-network`
- `persistence` is a crate of its own: its six lists and the tree of what pulls a unit in are the module's own description, and the switch between a list and a tree is a word in the view vocabulary rather than a type in the console
- `firewall` and `containers` are crates of their own: the ruleset screen and the containers screen are descriptions their modules hand over, and the console no longer keeps a list state, a sort table or a detail for either
- a section switched off in the configuration says so on every screen now, not only on the three that had the sentence written out
- `users` is a crate of its own too — its seven lists are panes the module describes, and a pane that would open onto nothing is no longer offered; the console reaches every section through one generic screen, so what is left of `bin/vigil` is the frame around it
- `ports` is the first subject to become a crate of its own — `crates/collectors/vigil-network` holds its reading, its parsers, its rules, its samples and its declaration; the daemon asks the module instead of pairing a collector with a rule set by hand
- `vigil-module` declares what a subject tells the two binaries about itself, and `vigil-view` the vocabulary a module describes its screen with — neither draws anything
- the crates are laid out by layer — `crates/core/` (vocabulary, ports, the differ), `crates/consumers/` (memory and output), `crates/collectors/` (a crate per subject watched); the folder is the layer, and the direction of a dependency is read from it


## [1.0.5] - 09/12/2026

### Fixed
- a finding closed by its pair is marked resolved on the console as it is in the store, instead of standing open until the daemon is restarted
- updated audit rule `auid>=1000`
- collector no longer calls itself degraded on a quiet host
- updated collector state after restart
- health is now taken right after a reading that found something
- a collector whose reading fails raises a finding, once, and a reading that goes through closes it
- a complaint a previous run left open is closed by the first reading that goes through, not by the greeting
- the record that loads the audit rule is no longer counted as somebody having run something


## [1.0.4] - 09/12/2026

### Added
- an outgoing buffer
- config for file reading
- the `containers` collector
- the `resources` collector
- `agent.collector.recovered` and `agent.buffer.drained`: the closing halves of
  `agent.collector.degraded` and `agent.buffer.dropping`

### Changed
- compaction of an NDJSON journal no longer reads back what it has just written
- the console is no longer told that nothing is buffered for sending
- the contract's agent-state message now carries `buffers` as a list where it carried
  one `buffer` object
- the console puts a list in order with `s` and narrows the findings with `f`
- the panel behind `d` on those sections prints every value the agent recorded about the row

### Fixed
- a finding closed by its pair is marked resolved on the console as it is in the store,
  instead of standing open until the daemon is restarted

## [1.0.2] - 09/11/2026

### Added
- reading of this host's firewall: the `firewall` collector over the nftables ruleset that
  `vigil-firewall.timer` writes to `/var/lib/vigil/firewall/ruleset.json` 
- a firewall section in the console
- `vigild collector <name> enable` and `disable` commands

### Fixed
- the summary screen lined the agent's fields up one column to the right wherever the host
  column had run out of rows

## [1.0.1] - 09/11/2026

### Fixed
- a release run over a tag that already has a release updates it instead of failing

## [1.0.0] - 09/11/2026

### Added
- project initialize

