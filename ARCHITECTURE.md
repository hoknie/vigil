# `vigil` Architecture

## 0. How to read this document

This document is a **set of rules** for the codebase. The rules in §3 apply to all new code and
to every file you modify. Where the tree does not yet comply, the gap is recorded as a ⚠️ row in
§9: it is debt to be paid down, and it gives no permission to repeat the pattern. Deliberate
exceptions are recorded in §10 together with their rationale; any violation missing from §10 is
a defect.

---

## 1. Core principles

Everything else in this document follows from four principles.

1. **A workspace with explicit module boundaries.** The product is a Cargo workspace of
   independently buildable crates: 17 libraries and 3 binaries. Crate boundaries are dependency
   boundaries, and the compiler enforces them.

2. **Three layers of code, with dependencies flowing in one direction:**
   - **io** — reading the host, storage, output; knows nothing about rules;
   - **decision** — the differ and the rules; pure functions over snapshots, no io;
   - **vocabulary** — types and the contract; depends on nothing.

   The direction is `io → decision → vocabulary`, and reverse dependencies are forbidden.
   The layering applies at **two scales**: across the workspace — `bin/* → crates/*`
   (binaries compose libraries, libraries know nothing about binaries); and inside
   `crates/*` — along the stages of the pipeline (§3.3): `collector → differ → rule → sink`.

3. **Separate deployment units along the "observe ↔ display" axis.** The daemon runs under
   systemd as root with a reduced capability set and **does not link `ratatui`**. The console
   reads the socket and **links none of the host readers, rules or storage**. The audit plugin
   is executed by `auditd`, holds none of the daemon's capabilities and knows nothing about
   findings or the socket. The three binaries have disjoint heavy dependencies and communicate
   only through a local unix socket and a spool file, each of them one-way.

4. **Quality is enforced by gates.** Formatting, linting and tests are fixed commands in the
   `justfile`, identical locally and in CI: `just check` (fmt · clippy `-D warnings` · test),
   `just docker-check` (the same gate on musl, the only environment where the Linux-specific
   half of the code compiles), and `just console-pty` (the console's keyboard handling under a
   real terminal). The current state of each gate is in §9.

---

## 2. Workspace topology

```
crates/core/              LAYER 1 — vocabulary and ports; knows nothing about any subject
  vigil-model/            VOCABULARY: finding, host, snapshot, contract, socket protocol
  vigil-config/           CONFIGURATION FILE: suppression entry, its block, safe file writes
  vigil-rules/            DECISION: rule ports, snapshot differ, rule set
  vigil-collect/          READING: Collector port, Health, CollectError, pure helpers
  vigil-module/           MODULE: Module port — how a subject presents itself to the binaries
  vigil-view/             VIEW: screen vocabulary — section, pane, column, row, detail
crates/collectors/        LAYER 2 — one crate per subject: reading, parsers, rules, view
  vigil-network/  vigil-users/  vigil-processes/  vigil-persistence/  vigil-firewall/
  vigil-resources/  vigil-containers/  vigil-files/  vigil-launches/
crates/consumers/         LAYER 2 — storage and output
  vigil-store/            STORAGE: Store port + file-based implementation + conformance suite
  vigil-report/           OUTPUT: Reporter port + sinks + output formats
bin/vigild/               DAEMON: composition root, scheduling, socket
bin/vigil/                CONSOLE: ratatui over the socket and over the view vocabulary
bin/vigil-audit-plugin/   PLUGIN: stdin from auditd → spool
```

**The folder is the layer, and the direction of a dependency is read from the folder path.**
`path = "../../core/vigil-model"` points down and is legitimate; `path = "../vigil-users"` from
a sibling module points sideways. A reviewer sees this in `Cargo.toml` long before the compiler
reports a cycle.

### A subject is a crate

Everything about one subject lives in one crate: `collectors/` (reading the host), `parsers/`
(decoding formats), `rules/` (decisions over the snapshot), `views/` (the screen, described as
data), `fixture/` (reference readings) and `modules/` (the declaration of the subject to the
daemon and the console). Spreading a single subject over several crates would require four
separate tables — a collector registry, collector-to-rules pairs, a screen list and a
finding-to-row map — and each of them can silently fall out of date.

**A module does not know its siblings.** If it needs another subsystem, the trait lives in
`core/` and the wiring happens in the composition root.

**`ratatui` is forbidden in modules.** A module hands over its view as data (`vigil-view`), and
only `bin/vigil` draws. A module that linked `ratatui` would pull the console into the daemon's
dependency graph; Cargo features are additive, so a feature flag cannot prevent that.

### Local crates and external dependencies

A crate is created for a subsystem of our own. A wrapper around a single external library stays a
plain dependency: if all of a crate's code is `pub use`, the crate should not exist.

### One crate per implementation

A second implementation of a port goes into the kind folder of the same crate (`sinks/`,
`stores/files/`). It becomes a **separate crate** only when it pulls in a heavy dependency that
every user of the port would otherwise have to build. A crate never wraps itself in a folder
repeating its own name.

### Dependency direction (strictly acyclic)

```
core/    vigil-model ◄── vigil-rules ◄── vigil-module ──► vigil-collect ──► vigil-model
         vigil-model ◄── vigil-view                            ▲
                                                               │
collectors/vigil-<subject> ──► core/{model, collect, rules, module, view}

consumers/vigil-store  ──► core/vigil-model
consumers/vigil-report ──► core/vigil-model

bin/vigild             ──► collectors/* · consumers/* · core/*
bin/vigil              ──► collectors/* (for their views only) · core/{vigil-model, vigil-view,
                           vigil-module, vigil-config}
bin/vigil-audit-plugin ──► collectors/vigil-launches
```

- `vigil-model` depends on nothing and has no io, no async and no network access;
- `vigil-config` depends on none of our crates and knows only the configuration file; findings,
  snapshots and screens are outside its scope (see §10 on its io);
- `vigil-view` depends only on `vigil-model` and knows no subject;
- `vigil-collect`, `vigil-store` and `vigil-report` are independent of one another;
- **modules are independent of one another, and so are features.** If a foreign subsystem is
  needed, declare a trait in the crate's own `ports/` and provide the implementation in the
  composition root;

---

## 3. Module and file organization rules

### 3.1. Module roots contain declarations only

**A module root — every `mod.rs` and every crate's `lib.rs` — contains only declarations:**
`mod`/`pub mod` and `use`/`pub use` (re-exports). **No logic**: no `struct`/`enum`/`trait`,
no `fn`/`impl`, no constants, no tests. All logic lives in named sibling files that the root
declares and, where needed, re-exports. Tests for a kind folder live in a `tests.rs` next to it.

**The only exception is a split impl (§3.6):** when a `<type>/` folder splits a large `impl`
of one type into concern files, its `mod.rs` holds the `struct` itself and the skeleton of the
trait impl, so that the child modules can see private fields. The exception does **not** cover
free functions, constants or tests.

Any `mod.rs`/`lib.rs` should give a map of its module within five seconds, without reading code.

### 3.2. One logical element per file

- **One detection rule per file** in `rules/<domain>/<rule>.rs`, plus a row in the table
  test `rules/tests.rs`.
- **One collector per file** in `collectors/<os>/<subject>.rs`. The shape is fixed: the
  collector struct plus `impl Collector for` it, and nothing else. Format decoding goes to
  `parsers/`; pure functions go to `helpers/`.
- **One external format per parser** in `parsers/<domain>/<format>.rs`: bytes in, domain
  records or a **named refusal** out. No io, no decisions.
- **One sink per file** in `sinks/`; one output format per file in `formats/`.
- **One console screen per file** in `screens/`.
- **One trait with its implementation per file**, with no free functions or unrelated
  structs in it. **An implementation is named after its trait with an `Impl` suffix** when the
  trait and the struct share a namespace; the suffix only disambiguates and carries no other
  meaning.
- **Types used by a trait belong to the crate.** A type that a trait only accepts or returns goes
  in `types/`, where a second consumer can use it without depending on the port. To decide, ask
  which part is the **requirement** and which part is the **shape of the answer**: the
  requirement stays a port, the shape of the answer moves to `types/`. The only acceptable
  neighbours of a port are test fakes under `#[cfg(test)]`.
- **One struct with its `impl` blocks per file**, together with its associated constants.
  Another struct means another file.
- **No free functions in a file containing a struct, trait or impl.** Pure functions without
  `self` go to `helpers/` or to a dedicated functions file for that concern.
- **Magic literals become named `const`s**, especially repeated ones and those in `match`
  arms. A constant lives next to what it means.
- **A closed vocabulary is built from named constants, and so is the list that declares it.**
  The array that declares a vocabulary is usually its definition: every element has a name, the
  list is built from those names, and consumers refer to the names. A typo in a constant name
  fails to compile; a typo in a literal compiles and leaves a case silently unhandled. In this
  codebase the closed vocabularies are `KnownKind`, `Severity` and `State`, each paired with an
  **open door** (`Kind::Unknown`) as described in §6.
- **An open vocabulary stays a string.** Enums are for vocabularies with a finite, defined set of
  words. Rule names, finding keys and labels are open vocabularies: give names to the subset you
  can justify, keep the rest as strings, and state this where the type is defined.
- **A vocabulary duplicated elsewhere is reconciled by a gate.** The JSON Schema is invisible to
  the Rust compiler: a word added to `KnownKind` and missing from
  `host-findings-v1.schema.json` produces records that a receiver will reject. The check lives
  in the drift test (`crates/core/vigil-model/tests/conformance.rs`) and in the conformance
  suite under `docs/contract/`.
- **A method takes at most 4 arguments; a free function takes at most 3.** Beyond that, group
  the arguments into a struct.

### 3.3. Pipeline stages (host → snapshot → finding → output)

The pipeline flows strictly in one direction:

```
collectors → Snapshot → diff → Change → rules → Finding → Store → Reporter
```

- **`collectors/`** answer "what is on the host right now", and nothing else: comparison with
  the past and judgement happen later. A failed read is reported as a `CollectError` and a
  `Health`; an empty snapshot always means the host really has nothing to report.
- **`diff`** answers "what changed", using only two snapshots. No io.
- **`rules/`** answer "is this worth reporting". A rule is a pure function from a `Change` (or
  from a batch of changes, for a `BatchRule`) to a `Finding` or to silence.
- **`Store`** answers "have we seen this before, and what did a person decide about it".
- **`Reporter`** answers "where does this go"; the content is already decided.

**Every event source must reduce to a snapshot or a delta.** A stream that bypasses the differ
detects the same event twice, and this rule exists to prevent that. The audit integration follows
it: the plugin writes a spool, and the collector reads the spool and returns a **cumulative
snapshot of launches**.

### 3.3.1. A kind of element is a folder. Always. Even for a single element

This rule is strict and has no exceptions for small cases.

- **Each kind lives in its own folder, named after the kind in the plural.** Types go in
  `types/`, ports in `ports/`, collectors in `collectors/`, parsers in `parsers/`, rules in
  `rules/`, sinks in `sinks/`, screens in `screens/`, view descriptions in `views/`, module
  declarations in `modules/`, pure functions in `helpers/`.
- **A single element still gets a folder.** A lone type still goes in `types/`, next to
  `store.rs`. The reason is practical: a flat file invites "just one more thing", and three
  edits later it holds types, conversions and helpers. A folder establishes from the start
  where the second element goes.
- **One element of a kind per file**, named after the element in the singular:
  `parsers/audit/arguments.rs`, `rules/accounts/new_account.rs`. Generic names such as
  `misc.rs` or `common.rs` are forbidden.
- **A kind folder is named after what it declares.** `parsers/` means decoding an external
  format; a file that reads `/proc` for data is a collector, even if it parses something
  internally.
- **A kind folder is named with a noun.** Verbs such as `serve/`, `watch/` and `configure/`
  describe actions; name the folder after its contents (`socket/`, `loops/`, `wizard/`).
- **`adapters/` is reserved and means exactly one thing: an implementation of a port.**
  Adapters live in the composition root, because only the root knows about every subsystem.
  The test is simple: **is there a trait this implements?** If there is none, the folder needs
  a different name.
- **A kind folder's `mod.rs` contains declarations only** (§3.1).
- **One kind per file.** Types stay out of `ports/`, parsers stay out of `collectors/`, file
  reading stays out of `rules/`. Code that fits two kinds is two elements and belongs in two
  files.
- **A kind folder contains only its own kind, and subfolders that again contain only that
  kind.** `collectors/helpers/`, `ports/types/` and `rules/helpers/` are violations; `helpers/`
  and `types/` are **siblings** of the other kind folders. A nested `collectors/helpers/`
  declares that those functions belong to collectors, so as soon as a rule or a screen needs
  them, it has to either reach through another layer or copy the code. The only legitimate
  nesting is **the same role, deeper**: `collectors/linux/`, `parsers/roster/`,
  `parsers/audit/`.
- **An overgrown kind folder is grouped by meaning, using subfolders of the same kind.** The
  threshold is roughly **12 files**: beyond that a flat list reads like an alphabetical index,
  and it becomes unclear where the next file belongs. Groups are named after a subdomain
  (`rules/accounts/`, `parsers/roster/`, `views/detail/`) and never after a Rust construct. The
  "one element per file" rule still holds: grouping adds a level above the files and keeps each
  file intact.
- **New kinds may be added when they represent an architectural role.** Folders named after
  Rust constructs or used as catch-alls are **forbidden**: `structs/`, `enums/`, `traits/`,
  `impls/`, `utils/`, `common/`, `misc/`, and `core/` used as a catch-all.

| Folder | Contains | Does NOT contain |
|---|---|---|
| `ports/` | requirement traits towards other subsystems, one per file | implementations, answer shapes (those go in `types/`) |
| `collectors/` | one file per subject; struct + `impl Collector` | format decoding, comparison with the past, decisions |
| `parsers/` | decoding an external format: bytes → records or a named refusal | io, network, decisions about the data |
| `rules/` | one file per rule: one change → one finding or silence | host reads, storage access |
| `services/` | application logic over its own domain (differ, rule set) | io, transport |
| `stores/` | storage implementations, one subfolder per backend | domain decisions |
| `sinks/` | one file per output: file, syslog, webhook | choosing the recipient (that is configuration) |
| `formats/` | output record formats: RFC 5424, an NDJSON line | opening sockets, delivery |
| `screens/` | one file per console screen | access to the host or to storage |
| `views/` | the screen described as data: section, panes, columns, cells, detail | drawing, `ratatui`, host reads |
| `modules/` | the module declaration: period, unit, settings, assembly of the collector and rules, finding anchors | the rules and the collector themselves (they live in their own kinds) |
| `types/` | domain values, their invariants and closed vocabularies | the wire format of a foreign protocol |
| `helpers/` | pure functions without `self` or state, at **crate level** | anything with state; placement inside `collectors/` or `ports/` |
| `loops/` | a periodic pass: what it does on each tick | who starts it and when (that is `boot/`) |
| `boot/` | step-by-step process assembly, one file per step | domain logic |
| `cli/` | argument parsing and output formatting | everything else |
| `fixture/` | reference readings and answers, one file per snapshot or answer kind | assertions (those go in `tests.rs`), io, domain decisions |
| `tests.rs` | unit tests for its kind folder, placed next to it | tests for a neighbouring kind |

**`fixture/` is a kind with a narrow purpose.** It exists on both sides of the wire
(`crates/collectors/vigil-<subject>/src/fixture/`, `crates/core/vigil-rules/src/fixture/`,
`bin/vigil/src/ui/fixture/`, `bin/vigild/src/socket/fixture/`) and contains **only** the
construction of a reference reading or answer: one file per snapshot, named after its subject
(`sockets.rs`, `accounts.rs`, `launches.rs`, `answers.rs`), with assertions in the adjacent
`tests.rs`. A producer-side fixture is built with the crate's **real builder**. A fixture written
as a literal can only agree with itself, and `golden/` exists to catch that.

**`crates/core/vigil-model/golden/` holds reference samples as plain data.** The directory
belongs to the vocabulary crate because all three sides link it. There are four families, each
answering a different question:

- `snapshots/<collector>.json` and `protocol/<answer>.json` — **shape**: key classes, field
  paths, sets of JSON types, whether a field is required. All three sides check them for
  equality;
- `readings/<collector>.json` — **a complete reading, values included**, as produced by the
  real builders. Only the producer checks them: consumer fixtures describe scenarios (a
  backdoor, a deleted binary, specific severities), and requiring document equality from them
  would erase those scenarios;
- `settled/<subject>.json` — **values with a single source of truth in the code**: a
  collector's period (as declared by its module), the capacity of the findings list. Each pin
  sits on a named field and records both the value and its source; a consumer checks the shape
  everywhere and the value of pinned fields.

The producer regenerates them with `just golden`. They are an internal surface: the daemon and
the console ship in the same package, so no third party relies on them. The only published
surface is `docs/contract/` (§8), which is versioned and follows the N/N−1 acceptance rule.

**Domain folders and kind folders follow different naming rules.** A domain is named after its
subject, in the singular (`spool/`, `identity/`, `config/`, `link/`, `contract/`, `finding/`,
`host/`, `snapshot/`, `protocol/`); a kind is named after its role, in the plural. A domain may
contain kinds; a kind may contain subdomains of the same role. A kind never contains another
kind, and a domain is never named with a verb.

### 3.3.2. Binary-level kinds: `boot/`

- **Each binary's composition root is its `boot/` folder.** `main.rs` is a thin shim
  (`crate::boot::run()`), and `lib.rs` contains only declarations and re-exports. The lib+bin
  layout is required: integration tests can only use a library crate, so everything that
  assembles the application lives in the library.
- **One file per assembly step, named after the step.** `boot/run.rs` holds **only the order of
  steps**; `boot/watches.rs` builds the list of watches from the modules; `boot/start.rs` starts
  the threads. The set of modules is declared in exactly one list per binary:
  `bin/vigild/src/modules/registry.rs` for the daemon and `bin/vigil/src/ui/sections.rs` for the
  console. `bin/vigild/tests/modules.rs` fails if the two lists differ from each other or from
  the crates under `crates/collectors/`.

### 3.3.3. The wire format lives in `protocol/` and stays there

The format in which the daemon talks to the console is transport. It is declared once
(`vigil_model::protocol`), and **domain types stay on their own side of it**: a console screen
works with a protocol answer and has no access to the `Store`; the daemon responds with a
protocol structure and keeps its internal state private.

**The network has no reverse channel into the agent, by design.** Everything that reaches the
daemon **from outside the host** is data: `host-findings/v1` defines no reverse channel, so this
guarantee comes from the contract itself.

**The local console protocol has exactly three verbs, each added by a decision of the owner's
own.** `kill` closes a socket that a person has marked on the ports screen, either by
signalling the process that holds it or by destroying the socket itself. The same verb stops a
program marked in the list of running programs, by signalling each of its processes found in
`/proc` at the moment of the request. It is enabled in `vigil.yaml`
(`killing.from_the_console`, `off` by default), read at start-up, and can be changed neither from
the console nor over the network. The daemon enforces the limits and refuses to act on: PID 1,
the agent itself, a key absent from the latest reading, a socket whose owner is unknown,
`destroy` for anything other than a TCP socket, and a program with no processes left. Every
attempt, successful or refused, produces a finding (`agent.socket.killed`,
`agent.socket.kill_refused`, `agent.process.killed`, `agent.process.kill_refused`).

`change` changes the accounts of this host that a person has marked or edited on the accounts
screen: a user is edited or deleted (its home stays), a group created, edited or deleted, a sudo
grant in `/etc/sudoers.d` edited or deleted (never one in `/etc/sudoers`), a key or the keys of an
account added, edited or taken away, a session ended. What each list offers is one table,
`AccountObject::changings`, which the console offers and the daemon enforces. It has its own
switch, `accounts.from_the_console`, `off` by default, so a host that may stop a program is not
thereby a host where the console may grant sudo. The daemon runs the distribution's account tools
by absolute path, checks a sudoers file with `visudo -cf` before it replaces the old one, and
refuses uid 0 deletion or locking, deleting the agent's own account or the gid 0 group, a key
absent from the latest reading, and any name, path, key line or rule that does not pass its
checks. Every attempt produces `agent.account.changed` or `agent.account.change_refused`.

`control` acts on what this host starts by itself, from the startup screen: a unit or a timer is
stopped, started, disabled, enabled, masked or unmasked through `systemctl` — those six words and
no seventh, each with its opposite beside it, so a person undoes from the same band what they
just did — and a cron line is commented out or brought back. `restart`, `reload`, `isolate`,
`daemon-reload`, `reboot` and the rest have no word in `Controlling` at all, so no shape of the
request reaches them. It has its own switch, `units.from_the_console`, `off` by default. The
daemon refuses a key absent from the latest reading, a unit name it did not read off this host or
that could be read as an option, the agent's own unit and the unit any of its readings needs, a
crontab whose line is no longer there byte for byte, and a job that is a whole file of
`/etc/cron.daily` and its neighbours. The crontab is rewritten the way an `authorized_keys` file
is, by a candidate renamed over it, following no symbolic link. Every attempt produces
`agent.unit.controlled`, `agent.unit.control_refused`, `agent.cron.changed` or
`agent.cron.change_refused`. A fourth verb
requires an equivalent decision, and the test
`three_requests_do_something_on_the_host_and_all_three_are_named_here` fails if one appears.

**The daemon's watch path starts no programs.** No collector, differ, rule, sink or socket answer
ever reaches `exec`. Exactly four places start a program, and each carries out an instruction a
person gave on this host: the operator command `vigild collector <name> enable|disable`
(`bin/vigild/src/collector/host.rs`, which runs `/usr/bin/systemctl` by absolute path), closing a
socket with `ss -K` after a person confirms it at the console
(`bin/vigild/src/killing/destruction.rs`, which runs the distribution's `ss` by absolute path),
changing an account a person saved or deleted at the console
(`bin/vigild/src/accounts/tools.rs`, which runs the distribution's account tools, `visudo` and
`loginctl` by absolute path, with an empty environment and no shell), and the word a person chose
on the band of the startup screen (`bin/vigild/src/units/systemctl.rs`, which runs `systemctl` by
absolute path with an argument list of exactly two: the word and the unit name). Incoming
messages trigger none of them.

Six tests in `bin/vigild/tests/exec.rs` enforce this:
`the_four_places_that_start_a_program_are_the_operators_command_the_kill_the_account_change_and_the_unit_it_asked_for`
(exactly four such files exist in the tree), `the_watching_loop_and_the_rules_reach_none_of_them`
(reading, the differ and the rules cannot reach them),
`the_place_that_starts_one_is_reached_from_the_command_line_and_from_nowhere_else`
(only the operator path reaches it),
`the_account_tools_are_reached_only_through_the_console_change_and_from_nowhere_else` (only the
console's `change` reaches them),
`the_unit_and_the_crontab_are_reached_only_through_the_console_control_and_from_nowhere_else`
(only the console's `control` reaches `systemctl` and a crontab) and
`the_collectors_themselves_start_nothing_whatever_they_have_to_read` (`exec` never appears in
the collectors; for this reason the firewall reading is produced by a packaged systemd timer).

### 3.3.4. A crate declares its own capabilities

Each crate declares what it can do. A module declares itself completely (`modules/`): its name,
subject, period, systemd unit, settings key, collector, rule set, finding anchors and console
section. `vigil-report` declares its sink names. The root assembles these declarations and
**fails at start-up** when a declaration and its implementation disagree.

As a consequence, the root contains **no name tables**. The collector list, the
collector-to-rules pairs, the numbered screen list and the finding-family-to-row map are all
derived from module declarations. A table that duplicates a declaration is where a new module
gets silently forgotten.

### 3.4. Re-export facade in the root `lib.rs`

Consumers use `vigil_model::Finding`; the internal path `vigil_model::finding::record::Finding`
is private detail. The path to a type is outside the contract, so moving files inside a crate
must leave its neighbours unaffected.

### 3.5. Comments in code

**`.rs` files contain no comments**: no `//`, no `///`, no `//!`. The decision was made while
preparing the code for publication and is enforced in code review.

Content that would otherwise be a comment goes elsewhere:

- **the reason for a decision goes into a test** that fails if the decision is reversed. The
  test name carries the explanation: `a_file_in_a_shape_we_do_not_know_is_not_an_empty_host`
  states the decision clearly, and it cannot go stale without the test failing;
- **prose for a reader of the code goes into the crate's `README.md`**: its purpose, the
  decision it follows from, its dependencies and a context graph;
- **prose for an operator goes into the systemd units, the conffiles installed from
  `packaging/audit/`, and `docs/contract/`** — places that are read without the source code.
  `config/vigil.example.yaml` lists the defaults and nothing more.

**Shipped text never mentions internal process.** Code, logs and the console contain no phases,
epics or document names: someone reading a log line has no idea what "stage 3" means, and six
months later the author will not remember either. References to internal documents are even
worse: those documents are kept out of git, so the reference points to a file that exists in no
clone.

### 3.6. File size, splitting `impl` blocks and structs, grouping by domain

§3.1 and §3.2 are necessary but not sufficient: "one trait + impl per file" still **permits** a
1,300-line file if it is a single `impl`.

**File size budget.** The target is ≤300 lines. A file above 400 lines is a code smell and needs
either decomposition or an explicit §10 entry with a rationale. Containing a single impl is no
justification for a huge file.

**Function size budget — hard limit of 300 lines.** There are no exceptions: a long function
always merges several questions into one, and the order in which they are answered becomes hard
to follow. Split **by cohesion**: place the boundary where the fewest values cross between the
pieces, and group those values into a struct (see "at most 4 arguments"). A step that may produce
the final result itself returns `ControlFlow<Answer, Continue>`. An identical tail repeated
across several branches is a missing operation and should be extracted under its own name.

**Splitting an `impl` by concern.** A large **inherent** `impl Type` is split across several
files as separate `impl Type { … }` blocks, one file per cohesive group of methods. Two Rust
constraints determine the layout:

- **a trait impl must stay in one block**, with all trait methods together. A large trait impl is
  therefore kept thin: its methods **delegate** to inherent methods in concern files;
- **private fields are visible only to the type's module and its descendants.** Concern files
  that read private state must be **child modules** of the type: a `<type>/` folder with a
  `mod.rs` (the `struct` itself plus the trait impl skeleton) and `<concern>.rs` children. Flat
  sibling files require `pub(crate)` fields, which is acceptable only where the field is shared
  anyway.

In short, §3.2 means **one cohesive unit per file**.

**Decomposing structs.** A struct with more than 7 fields, or one that mixes concerns, becomes
a composition of sub-structs, each in its own file. Fields usually cluster by subsystem: a console
screen has "what is shown", "where the cursor is", "what is being searched for" and "what is
collapsed" — four structs in place of fifteen fields.

**Promoting to a domain folder.** When three or more sibling files belong to one domain, they
move into a subfolder with a declarations-only `mod.rs`. Such folders are named **after the
domain** (`accounts`, `keys`, `roster`) or **after a role** from the §3.3.1 table, and
**never** after a Rust construct.

### 3.7. Heavy work stays off the watch path

**The watch pass never performs work whose duration depends on a third party.** Reading the
host never waits for a sink's network, a receiver's disk or a syslog response: delivery runs on
its own thread with a **bounded queue**, and on overflow the oldest entries are dropped so that
observation continues. "Heavy" has an objective definition: work whose cost grows with data
(number of processes, spool size, files in a tree) is unbounded, because the host decides how
much data there is.

It follows that:

- **each collector has its own period**, so an expensive collector runs less often than a
  cheap one;
- **each pass has a ceiling**: the spool is read up to `CEILING_BYTES`, history is trimmed to its
  limit, tree walks are bounded by depth and file count;
- **every interaction with a third party has an explicit timeout.** A call without a timeout can
  wait forever;
- **the product has no async runtime.** This is a measured decision: the agent runs a handful of
  threads, and a runtime would add dependencies, static binary size and build time on the
  customer's host (§6, "Measure first").

**Resource budgets.** These limits are requirements: the agent runs on the customer's host, and a
customer disables a resource-hungry agent first.

| What | Budget |
|---|---|
| CPU at idle | ≤1% of one core |
| RSS | ≤64 MB |
| Local history | bounded by its limit; on overflow the oldest entries are dropped |
| Spool read per pass | `CEILING_BYTES`; anything beyond is recorded as dropped |
| Collector failure | disables that collector; the process keeps running |
| Sink failure | findings are buffered and watching continues; running with no sinks is normal |

Every change to a collector, the differ or the pass includes measurements taken before and after.

---

## 4. Binary templates

The general composition-root rule is §3.3.2; this is how it applies to each of the three
binaries.

- **`vigild`** is the only component that knows about everything: it reads the configuration,
  assembles collectors from the declared modules, builds the rule sets, opens the store, creates
  the sinks, starts the scheduler and listens on the socket. Any failure at start-up is reported
  loudly: an agent that reads nothing must never start silently.
- **`vigil`** knows nothing about `/proc`, rules or storage files. Its only connection to the
  outside world is the socket client; everything else is screens over the answers it receives.
  This also makes the console testable without a host in a particular state.
- **`vigil-audit-plugin`** is the smallest and most restricted binary: stdin → spool, with no
  interpretation, no findings and no socket. Another daemon executes it, so its minimal feature
  set is also its minimal attack surface.

---

## 5. Two kinds of building blocks

- **A library crate** is a reusable core exposing a port (`vigil-collect`, `vigil-store`,
  `vigil-report`). It is unaware of who assembles it and must be testable in isolation.
- **A domain inside a crate** is a set of §3.3 layers around one subject (`spool/`,
  `identity/`, `config/`). It is unaware of neighbouring domains and communicates with them
  through the root.

If testing a block requires a host in a particular state or a live neighbour, the block has the
wrong shape.

---

## 6. Key patterns

- **Ports for alternatives.** Wherever alternative implementations are expected, there is a
  trait: eBPF will be another `Collector`, redb another `Store`, the contract receiver another
  `Reporter`. An `if` on the backend inside business logic signals a missing port.
- **Snapshots for every source** (§3.3).
- **"Could not read" and "empty" are distinct results.** Every collector has a `Health`, and
  every snapshot element carries an incompleteness marker. A silent zero reads as "no attack",
  which is the worst possible answer.
- **An open door in a closed vocabulary.** `Kind::Unknown`, `Severity::Unknown`,
  `CollectorState::Unknown`: a receiver one version behind must **display** a value it does not
  recognise and keep the record.
- **Redaction happens before writing.** Secrets live in `cmdline` and in `ExecStart`; they are
  removed before anything reaches memory or disk, because whatever reaches the buffer has already
  leaked. Redacted fields are listed in `Finding::redacted`, so a reader sees that a value was
  hidden.
- **Tests exercise the real logic.** A rule is tested against a fixture; storage is tested against
  a conformance suite that every implementation of the port must pass. No test needs a host in a
  particular state.
- **Measure first.** Anything that affects cost comes with a number: a change to a
  collector or the differ includes before-and-after measurements; a new dependency includes its
  transitive crate count, binary size impact and clean build time.

---

## 7. Identifiers and data

- **`host_id`** is a keyed hash of `/etc/machine-id`; the raw value never leaves the host. **The
  field name and the file name differ on purpose:** "host" is what the whole product calls the
  node it watches, while `machine-id` is the kernel's file name and cannot be renamed. A test in
  `bin/vigild/src/identity/host.rs` fails if the path is "corrected".
- **`install_id`** is generated on first start and persisted in the state directory. **A clone
  carries the identity of its source:** an image captured after installation brings the same
  pair of identifiers, and that pair is how a clone is detected. The agent has to report it at
  first contact, well before any incident.
- **`event_id`** is a UUIDv7: the embedded timestamp lets records sort without a separate field
  and makes duplicates visible to the receiver.
- **`finding_key`** identifies what a finding **is**, independent of when it happened. Key and
  kind together form a history: two kinds on the same key (`port.listen.new` and
  `port.listen.removed`) are two separate histories and must stay separate.
- **Local data stays within its budget**, and files are created with an explicit mode of 0600,
  independent of the process `umask`.

---

## 8. The `host-findings/v1` contract

The product's only published surface is `docs/contract/`: the JSON Schema and the conformance
suite (samples that must be accepted and samples that must be rejected). Rules for changing it:

- the contract and `vigil-model` change **together**, in the same change;
- **MINOR** for additive changes; **MAJOR** for everything else;
- **a receiver accepts versions N and N−1** (`SchemaVersion::accepts`), so 1.x must accept 0.x
  and must reject 3.0;
- every such change includes its new conformance samples;
- a field present on one side and missing on the other is the failure this discipline is designed
  to expose.

---

## 9. Conventions and quality

| Aspect | Decision | Status |
|---|---|---|
| Edition / resolver | `2024` / `3` via `[workspace.package]` | ✅ |
| Toolchain | `rust-toolchain.toml`, channel pinned; the same version in CI and in the development container image | ✅ |
| Product language | English everywhere: code, logs, console, `CHANGELOG.md`, commit messages | ✅ |
| Abbreviations in names | forbidden; write `organization_id` in full | ✅ |
| Style | `cargo fmt --all --check` | ✅ |
| Linter | `cargo clippy --workspace --all-targets -- -D warnings` | ✅ |
| Tests | `cargo test --workspace`; no host state, no network | ✅ |
| Linux-specific code | `just docker-check` — the same gate on musl | ✅ |
| Console keyboard handling | `just console-pty` — under a real terminal | ✅ |
| README in every crate | purpose, decision, dependencies, graph | ✅ |
| Comments in `.rs` | none (§3.5) | ✅ zero |
| `#[allow(...)]` in the tree | forbidden; fix the cause of the warning | ✅ zero |
| `main.rs` is a shim, the root is in `boot/` | all three binaries | ✅ |
| Module roots contain declarations only | `mod.rs`/`lib.rs` without logic or tests | ✅ |
| A kind is a plural folder | `ports/`, `parsers/`, `collectors/`, `rules/`, `sinks/`, `types/`, `helpers/` | ✅ |
| Kind folders are nouns | `socket/`, `loops/`, `wizard/` | ✅ |
| Grouping an overgrown kind | >12 files ⇒ subfolders of the same kind | ✅ no folder above 12 files |
| File size budget | ≤300 lines, >400 is a smell | ✅ the longest `.rs` file is 298 lines |
| Function size budget | hard limit ≤300 lines | ✅ |
| Measurement for cost-affecting changes | before-and-after numbers | ✅ |

---

## 10. Deliberate exceptions

Each exception is recorded as a row with its rationale. An empty table is better than an
unrecorded violation.

| Where | What | Why |
|---|---|---|
| `crates/collectors/vigil-<subject>/src/fixture/`, `crates/core/vigil-rules/src/fixture/`, `bin/vigil/src/ui/fixture/`, `bin/vigild/src/socket/fixture/` | the kind is named in the singular, contrary to §3.3.1 | The name predates the definition of the kind and is shared by all sides of the wire: `fixture::` appears in more than sixty console source files. Renaming is a purely mechanical edit, and combining it with a substantive change would hide that change in noise. The rename should land as a standalone edit; until then the debt is recorded here. |
| `crates/core/vigil-config/src/helpers/write.rs` | io in `core/`, the only io in that layer | The crate exists to own a **file on disk**. Both programs write suppressions (`vigil suppress` and a key on the findings screen), and `vigild configure` and `vigild collector` write the same file. Without a shared crate, the same 0600 write ("write to a temporary file, rename it, keep the previous version as `.previous`") would live in two binaries that must stay byte-identical and would drift apart on the first edit. Its dependencies keep the boundary tight: `vigil-config` links none of our crates, performs exactly one io operation (`write`), and is invisible to `vigil-model` and `vigil-rules`. |

---

## 11. Recipes

**Add a detection rule.** Create `crates/collectors/vigil-<subject>/src/rules/[<domain>/]<name>.rs`
with a struct and `impl Rule`; add a line to the folder's `mod.rs`; add it to the subject's rule
set in `rules/set.rs` (`<subject>_rules()`); add a row to the table test `rules/tests.rs` (`rules/tests/table.rs` where the rule tests are a folder); add a
variant to `KnownKind` if the kind is new — and, for a new kind, update `docs/contract/` and add a
sample to the conformance suite.

**Add a subject (collector).** Create a crate under `crates/collectors/vigil-<subject>/` and add
it to the workspace members. Inside it: `collectors/linux/<subject>.rs` with a struct and
`impl Collector`; format decoding in `parsers/[<domain>/]`; rules in `rules/`; the screen as data
in `views/`; the declaration in `modules/<subject>.rs` with `impl Module` (name, subject, period,
unit, settings key, collector, rules, section, families). Register the module in both
`bin/vigild/src/modules/registry.rs` and `bin/vigil/src/ui/sections.rs`, at the same position.
Include a test for incomplete reads: "the file does not exist" and "the file cannot be read" are
different results.

**Add a parser.** Create `parsers/<domain>/<format>.rs`: bytes → records or a named refusal. No
io, no decisions. The first test covers an unrecognised format.

**Add a sink.** Create `sinks/<name>.rs` with a struct and `impl Reporter`; a sink-specific output
format goes in `formats/`; add a configuration section; set an explicit timeout; on failure the
sink buffers and watching continues.

**Add a console screen.** A subject's section is described as data in its crate's `views/`; a
screen that belongs to the console itself goes in `bin/vigil/src/ui/screens/<name>/`. Keyboard
navigation follows the three-level focus model (tabs → list → details): `→`/Enter moves in,
`←`/Esc moves out. Verify it with `just console-pty`.

**Split an overgrown file.** Identify cohesive groups of methods; move them into concern files as
**child modules** of the type (`<type>/mod.rs` plus children); keep the trait impl thin and
delegating. The crate's public facade stays unchanged; a facade change belongs in a separate
change.
