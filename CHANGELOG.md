# Changelog

All notable changes by release tag, newest first.
Format:
- [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
- [SemVer](https://semver.org/spec/v2.0.0.html)

## [1.0.10] - 09/--/2026

### Added
- vigild runs on macOS as a launchd service, with its own paths, a universal `.pkg` and `.tar.gz`
- macOS collectors: network, processes, users, resources, files, persistence (launchd jobs),
  firewall (pf and the Application Firewall), launches (eslogger), container engines
- `vigil-firewall-dump` and `vigil-launches-spool`: the programs launchd runs on a Mac
- `persistence.launchd.new` (contract 1.8)
- `just distributions` runs the agent on eleven Linux distributions in containers
- `just coverage`, `just coverage-html`, `just docker-coverage`

### Changed
- a collector with no reader for this system starts the daemon as unavailable instead of stopping it
- on macOS the console refuses account changes and launchd jobs, with the reason
- the GitHub workflow caches cargo builds and runs on `main` and pull requests; Dependabot added

### Fixed
- user crontabs of openSUSE and Alpine, `/etc/cron.d` files cronie runs, Alpine's periodic and
  `local.d` scripts, openSUSE's `/usr/etc` profiles and sudoers with their includes
- the audit log is read where `auditd.conf` puts it
- the state directory is made `0700` on a host installed from the binaries alone
- a host that keeps `os-release` only under `/usr/lib`, or names a build and no version

## [1.0.9] - 09/18/2026

### Added
- collectors are read from `collectors_path` in `vigil.yaml`
- `vigild collector <name> enable|disable` edits that block in place, or writes a file for a
  collector that has none
- files: the watched paths live in a watch list of their own
- reporters can be kept apart from `vigil.yaml` the same way, under `reporters_path`
- the findings section has two lists, `reported · silenced`
- the findings filter offers each kind the agent holds, with how many findings it would leave
- findings sort by SEEN and by FIRST SEEN

### Changed
- the socket collector is renamed `ports` → `network`
- the console refusals of killing, accounts and units name the key and the collector file it lives in

### Fixed
- a container engine named in `engines:` that is not installed on the host no longer marks the
  containers-engines reading degraded

## [1.0.8] - 2026-09-18

### Added
- `env/scripts/install.sh` installs and updates vigil from a release, as a package
- the images, volumes, networks, projects, pods, secrets and registries of docker and podman

### Fixed
- `just golden` no longer fails a test that reads a sample while another test writes it

## [1.0.7] - 2026-09-17

### Added
- launches show the last run beside the run count, sort by either, and H opens the runs of a row
- units and timers can be stopped, started, disabled, enabled, masked and unmasked
- cron lines can be commented out and back in
- the firewall section lists tables and zones, says what each rule matches and does
- the filesystems of this host are drawn as a tree of the disks and volumes they sit on
- watched paths are edited in `vigil.yaml`, from the console or by hand

### Changed
- console look: rounded frames, inset fields with a real cursor, dropdowns and popups
- console look: a section's panes carry the frames, the popup shadow dims instead of covering

### Fixed
- the help page fits an 80x24 terminal

## [1.0.6] - 2026-09-14

### Added
- `vigil suppress` writes, removes and lists suppressions
- findings can be silenced from the console
- picking rows on the findings screen
- sockets can be closed and silenced from the console
- programs can be stopped from the console
- the detail pane no longer takes the focus when it opens
- the start-up lines say whether the console of this host may ask the agent to close a socket,
  and name the key that decides it
- accounts can be changed from the console

### Changed
- the configuration path the daemon was started with is the one the console edits
- the screen for a reading no section draws
- a pane's name is its own, not a fixed list
- `vigil.yaml` is read in two passes: the daemon's own keys, then the key each module names as
  its own
- crates reorganized: one crate per subject watched
- keys changed for a clearer console
- lists draw faster

### Fixed
- one object silenced twice is written down once
- moving through a list of several hundred rows no longer lags: only the rows on the screen are
  drawn
- a socket state between two readings of an unchanged host
- the record that loads the audit rule is no longer counted as somebody having run something

## [1.0.5] - 2026-09-12

### Fixed
- updated audit rule `auid>=1000`
- collector no longer calls itself degraded on a quiet host
- updated collector state after restart
- health is now taken right after a reading that found something
- a collector whose reading fails raises a finding, once, and a reading that goes through closes it
- a complaint a previous run left open is closed by the first reading that goes through, not by the greeting
- the record that loads the audit rule is no longer counted as somebody having run something

## [1.0.4] - 2026-09-12

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

## [1.0.3] - 2026-09-11

### Added
- the agent, the console, the `host-findings/v1` contract and the packages
- reading of this host's firewall: the `firewall` collector over the nftables ruleset that
  `vigil-firewall.timer` writes to `/var/lib/vigil/firewall/ruleset.json`
- a firewall section in the console
- `vigild collector <name> enable` and `disable` commands

### Fixed
- a release run over a tag that already has a release updates it instead of failing
- the summary screen lined the agent's fields up one column to the right wherever the host
  column had run out of rows

## [1.0.1] - 09/11/2026

### Fixed
- a release run over a tag that already has a release updates it instead of failing

## [1.0.0] - 09/11/2026

### Added
- project initialize

