# vigil-users

> Who may log in to this host, as whom, and with what.

## Why

Accounts, groups, sudo grants, SSH keys and sessions are read from the files the system keeps
them in. A new account, an unlocked one, a key added to `authorized_keys` or a member added to a
privileged group each becomes a finding.

The console can edit and remove accounts, groups, grants in `/etc/sudoers.d`, keys and sessions.
That ability is off until the operator switches it on in this block, and every attempt is itself
a finding.

## What's inside

| | |
|---|---|
| **Reads** | `/etc/passwd`, `/etc/group`, `/etc/shadow`, sudoers, SSH keys, sessions |
| **Raises** | `user.*` |
| **Every** | 300 seconds |
| **Settings** | the `users` block, shipped in `config/collectors/users.yaml` with a comment above every key |

## Depends on

- `vigil-model` · `vigil-collect` · `vigil-rules` · `vigil-module` · `vigil-view`
- `serde_json`; `libc` on macOS

## Where it fits

```text
vigil-model   vigil-collect   vigil-rules   vigil-module   vigil-view
     └──────────────┴──────────────┼──────────────┴──────────────┘
                                   ▼
                              vigil-users
                                   │
                   ┌───────────────┴───────────────┐
                   ▼                               ▼
          vigild · reads and judges        vigil · draws its screen
```
