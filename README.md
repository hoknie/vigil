# vigil

**Host protection agent.** It watches the host it runs on — listening sockets, accounts and
logins, processes, what people run, file integrity, persistence, containers, resources —
compares each reading with the previous one, and reports what changed.

Components:

- **`vigild`** — reads on a schedule, keeps findings in a local database, sends
  them wherever it is configured to — including nowhere.
- **`vigil`** — a terminal interface over the daemon's local socket, for the
  case it was built for: an incident on a host that ssh reaches and nothing else does.
- **`vigil-audit-plugin`** — `auditd` starts it and feeds it events
  on stdin, and it appends the ones about program launches to a file the daemon reads. It holds
  none of the daemon's capabilities and can do nothing to the host.

## Install

```bash
dpkg -i vigil_0.1.0_amd64.deb        # Debian, Ubuntu
rpm -i vigil-0.1.0-1.x86_64.rpm      # RHEL, Rocky, AlmaLinux, Fedora

vigild configure --dry-run           # what this host can watch
systemctl enable --now vigild        # enable and start vigil
```

### Building the packages

```bash
just package          # all binaries into ./dist
just package-verify   # install them on clean debian and almalinux hosts and check
just package-unit     # systemd-analyze verify over the installed unit
just package-systemd  # the unit under a real systemd; measures what its hardening allows
```

## Build and run

```bash
cargo test --workspace          # test the project
cargo run --bin vigild -- config/vigil.example.yaml
cargo run --bin vigil -- ui     # the console
cargo run --bin vigil -- capture # print info for a script or a pipe
```
