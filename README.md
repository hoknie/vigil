# vigil

**Host protection agent.** It watches the host it runs on — listening sockets, accounts and
logins, processes, what people run, the firewall, file integrity, persistence, containers,
resources — compares each reading with the previous one, and reports what changed.

Components:

- **`vigild`** — reads on a schedule, keeps findings in a local database, sends
  them wherever it is configured to — including nowhere.
- **`vigil`** — a terminal interface over the daemon's local socket, for the
  case it was built for: an incident on a host that ssh reaches and nothing else does.
- **`vigil-audit-plugin`** — `auditd` starts it and feeds it events
  on stdin, and it appends the ones about program launches to a file the daemon reads. It holds
  none of the daemon's capabilities and can do nothing to the host.
- **`vigil-firewall.timer`** — a systemd timer that runs `/usr/sbin/nft --json list ruleset`
  and writes the output where the daemon reads it. The daemon starts no program of its own, so
  the privilege that reading the ruleset needs lives in a unit you can read and
  `systemctl mask`.

## Install

```bash
dpkg -i vigil_0.1.0.debian.amd64.deb    # Debian, Ubuntu
rpm -i vigil_0.1.0.el.x86_64.rpm        # RHEL, Rocky, AlmaLinux, Fedora
tar -xzf vigil_0.1.0.linux.x86_64.tar.gz  # anywhere else: the binaries and the unit

vigild configure --dry-run           # what this host can watch
systemctl enable --now vigild        # enable and start vigil
```

### Building the packages

```bash
just package          # all binaries into ./dist
just package-verify   # install them on clean debian and almalinux hosts and check
just package-unit     # systemd-analyze verify over the installed units
just package-systemd  # the unit under a real systemd; measures what its hardening allows
```

## Build and run

```bash
cargo test --workspace                                  # test the project
cargo run --bin vigild -- config/vigil.example.yaml     # daemon start
cargo run --bin vigil -- ui                             # the console
cargo run --bin vigil -- capture                        # print info for a script or a pipe
```
