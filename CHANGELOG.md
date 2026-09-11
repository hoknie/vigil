# Changelog

All notable changes by release tag, newest first.
Format:
- [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
- [SemVer](https://semver.org/spec/v2.0.0.html)

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

