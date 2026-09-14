# killing

The one thing this daemon does to a host it did not set up: close a listening socket that a
person marked at the console of that host and confirmed.

## The decision it follows from

`docs/designs/2026-09-14-DESIGN-console-kill.md` (owner, 2026-09-14), which reverses two
recorded rules — "no verb appears in the protocol that does something on the host" and "the
agent does not kill processes" — and says what was bought with them: a journal. Every attempt
becomes a finding, so a closed port is answerable at an incident. The two paths that do not
leave one were rejected there.

Off unless `killing.from_the_console` is `true` in `vigil.yaml`, read once at start-up.

## What is here

- `targets.rs` — a socket key becomes something to aim at, or a refusal with a reason in it.
  Every boundary is here, on the side that acts, because the console can draw anything: pid 1
  is never signalled, the agent never signals itself, a key that is not in the reading the
  daemon holds is refused by name (between the mark and the confirmation a port can close and
  another process take it), a socket whose owner was never resolved has nothing to aim at, and
  `destroy` on anything but tcp says what would close it instead;
- `signal.rs` — `SIGTERM` or `SIGKILL` through `rustix`, and what the kernel's refusal means in
  words an operator can act on;
- `destruction.rs` — closing the socket and leaving the process up, through the distribution's
  own `ss -K` by absolute path. One of the two places in this daemon that start a program, both
  listed in `tests/exec.rs`;
- `run.rs` — the whole ask: a key named twice is acted on once, and everything past the ceiling
  of 64 arrives with a sentence rather than disappearing;
- `report.rs` — `agent.socket.killed` and `agent.socket.kill_refused`, keyed by the socket so a
  repeat raises a counter instead of a second finding.

## Dependencies

`vigil-model` for the vocabulary and the protocol types, `vigil-network` for `SocketView` —
the crate that owns the shape of a ports reading — and `rustix` for the signal. Nothing here
reaches the store or the reporters: the findings are handed to the socket state, and the watch
cycle picks them up.

## Context

```
console ──kill──► socket/kill.rs ──► targets.rs ──► signal.rs
                       │                      └──► destruction.rs ──► ss -K
                       ▼
                  report.rs ──► State ──► loops/round/killings.rs ──► store · reporters
```
