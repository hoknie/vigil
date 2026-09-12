# Changelog

All notable changes by release tag, newest first.
Format:
- [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
- [SemVer](https://semver.org/spec/v2.0.0.html)

## [1.0.5] - 09/--/2026

### Fixed
- a finding closed by its pair is marked resolved on the console as it is in the store, instead of standing open until the daemon is restarted
- updated audit rule `auid>=1000`
- collector no longer calls itself degraded on a quiet host
- updated collector state after restart
- health is now taken right after a reading that found something


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

