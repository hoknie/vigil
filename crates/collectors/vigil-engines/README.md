# vigil-engines

What the container engines of this host hold: images, volumes, networks, containers, compose
projects, podman pods and secrets, and the registries the engines are allowed to pull from.

## The decision it follows from

The daemon does not start programs (`ARCHITECTURE.md`, trap 2 in `CLAUDE.md`), and talking to
`/run/docker.sock` is root on this host in one step. So the privileged half lives in a unit an
operator can read and `systemctl mask`: `vigil-containers.timer` runs
`/usr/sbin/vigil-container-dump`, which runs the engine clients and writes what they printed to
`/var/lib/vigil/containers/<engine>.json`, `0600`. This crate reads those two files on the
daemon's round — exactly the shape `vigil-firewall.timer` and `nft` already have
(`docs/designs/2026-09-17-DESIGN-containers-engines.md`).

The dump has no policy in it. Projection, stabilisation and the hiding of secrets all happen
here, where a fixture can pin them.

## The reading

One snapshot, source `containers-engines`, keyed `<engine>|<subject>|<id>`:

```
docker|image|sha256:18ad9bdc4c87      podman|network|podman1       docker|project|shop
docker|volume|shop_database           podman|pod|tools             docker|registry|registry.local:5000
docker|container|shop-web-1           podman|secret|shop-database-password
docker|engine|docker                  podman|engine|podman
```

The engine is the first segment because the containers screen selects an engine first and a
subject second; `class_of` reads that first segment, so a class of this reading is an engine of
this host. Every row also carries `subject`, so a row says what it is without its key.

Only what stands still on a still host is kept. Two passes over one dump build the same
reading, down to the value, and a test says so: `Status`, `State`, `RunningFor`, `CreatedAt`,
`Restarts`, `Size` of a container, the number of containers on an image, podman's `uptime` and
docker's `SystemTime` are all dropped. A counter in a reading is a finding every minute, and a
finding every minute is a product nobody reads.

| Subject | Kept |
|---|---|
| `engine` | `present`, `read`, `dump_read`, `unanswered`, version, storage driver, root directory, cgroup driver and version, logging driver, `rootless`, security options |
| `image` | id (normalised to `sha256:…`), `tags` (one row per image, not per tag), `untagged`, `digest`, `size` |
| `volume` | name, driver, mountpoint, `device` (the host path a bind volume points at), scope, labels, project |
| `network` | name, id, driver, interface, `internal`, `ipv6`, `subnets`, labels, project |
| `container` | name, id, image, image id, `networks`, `host_network`, `mounts`, `mounts_truncated`, `ports`, `pod`, labels, project, service |
| `pod` | name, id, infra id, cgroup, the containers in it, networks, labels |
| `secret` | name, id, driver, created and updated time, `value_redacted` |
| `project` | name, services, containers, working directory, compose files |
| `registry` | host, `insecure`, role (`configured`, `mirror`, `search`), the file it was read from |

Two gaps are the command list's, not an oversight. `docker network ls` prints no subnet and
`docker ps` prints no restart policy; both need `inspect`, which this dump does not run, so a
docker network carries an empty `subnets` list and no row carries a restart policy. `docker ps`
also truncates a long mount path with an ellipsis, and the row says `mounts_truncated` rather
than pretending the path is what was printed.

`dump_read` says whether this engine's dump could be read at all, and `unanswered` names the
subjects whose command failed or timed out, plus `registry` when the registries file could not
be read. They are what tells a subject with no rows from a subject nobody answered for, and both
the rules and the screen read them (`types::Standing`).

## Secrets

Nothing is copied from an engine's output that this crate did not name: every row is built
field by field from the table above, so a field an engine adds tomorrow cannot arrive by
accident. On top of that:

- **no environment is ever asked for.** `inspect` is the command that prints a container's
  environment, and it is not in the list any engine is asked (a test names it);
- **the command line is dropped.** `docker ps` prints it, and `mysql -p…` lives there;
- **a label whose name reads as a secret** (`password`, `token`, `secret`, `api_key`, `auth`,
  `credential`, `private`, `passphrase`) keeps its name and loses its value to `[redacted]`.
  The row carries `labels_redacted: true` so a reader sees "hidden" and not "empty", and the
  rules turn that into `/after/labels` in `Finding::redacted`;
- **a secret is a name and a time.** `secret ls` never prints a value and this crate never
  looks for one; `value_redacted: true` is on the row to say so;
- **`~/.docker/config.json` is never opened.** It holds the credentials this host pushes with.
  The registries are read from `/etc/docker/daemon.json` and `/etc/containers/registries.conf`,
  and a test fails if either path leaves `/etc`.

## Health

| What went wrong | What the console shows |
|---|---|
| neither dump file is there | `Unavailable` — the timer has never run or is masked, and `systemctl enable --now vigil-containers.timer` is in the line |
| neither dump file can be read | `Unavailable` — the directory is `0700 root:root` |
| a dump is not the document this build reads | `Degraded`, naming the file; the reading fails rather than publishing an empty host |
| a dump is larger than 16 MiB | `Degraded` (`CollectError::Budget`) |
| a dump is older than `dump_seconds` × 2 | `Degraded`, saying how old; the reading is still taken, because an old answer is still the last one known |
| the engine is not installed | `Ok` — an answer, not a failure: the reading has a row for it with `present: false`, and its lists say "not installed on this host". A host where no engine named in `engines:` is installed is `Ok` too, and is a host with no container engines |
| the engine answered some commands and not others | `Degraded`, naming the subject and what the engine said; the subjects that failed contribute no rows |
| a registries file cannot be read | `Degraded`, saying which registries are unknown |

An engine that is not installed and an engine that did not answer are different facts and are
never turned into one another (trap 1).

## The rules

Every finding is keyed `engine|<the row's own key>` — `engine|docker|image|sha256:18ad9bdc4c87`,
`engine|podman|volume|etc_backup` — so the family is `engine` and `Module::row_of` cuts it off
and lands on the row of this reading. The kinds are `container.<thing>.<what happened>`, as
`persistence.cron.new` is.

**What comes and goes** (`rules/lifecycle.rs`, one rule per subject, each switched by its key
under `report:`). A row that appears is `.new`, a row that goes is `.removed` and closes `.new`
under the same key, and a row that stays is `.changed` only when a field that matters moved:

| Subject | Kinds, severity | Fields that make it changed |
|---|---|---|
| image | `container.image.new` / `.removed` low, `.changed` medium | none on one row: an image is its id. A tag that left one image and landed on another in the same round is one `.changed` about the image it landed on (`rules/tag_moved.rs`), and neither half is also reported as new or removed |
| volume | `container.volume.new` / `.removed` / `.changed` low | `driver`, `mountpoint`, `device` |
| network | `container.network.new` / `.removed` low, `.changed` medium | `driver`, `subnets`, `internal` |
| compose project | `container.project.new` / `.removed` / `.changed` low | `services` — a project that recreated its containers under new names is the same project |
| pod | `container.pod.new` / `.removed` / `.changed` low | `containers`, `networks` |
| secret | `container.secret.new` / `.removed` / `.changed` medium | `id`, `driver`, `updated_at` — a secret written again under the same name is a new value |

Containers and registries have no rule of this kind: a container comes and goes with every
deploy, and the reading of `/proc` already watches what runs.

**Dangerous settings** — reported when they arrive or when a row turns into one, never again
while they stand, and never switched off by `report:`:

| Kind | Severity | When |
|---|---|---|
| `container.network.host_mode` | high | a container on the network of this host |
| `container.volume.host_mount` | high, critical for `/` | a volume whose `device` is a path of this host, or a docker container mounting one; the list of paths is `vigil_rules::is_a_path_of_this_host`, the one the `/proc` reading of the containers judges by |
| `container.image.untagged_in_use` | medium | a container that names its image by an id, which is how an engine prints an image no tag names |
| `container.registry.insecure` | high | a registry the engine may reach without TLS |

A mount docker cut short at fifteen characters is judged only where every ending of what is left
would be a path of this host (`/etc/ngi…` is, `/var/lib/docke…` is not). Podman's `ps` prints
where a mount lands inside the container, not where it comes from, so a podman container's
mounts are not judged; what podman binds is judged on its volumes.

**Two kinds stay unproduced**, and a test says so: `container.project.privileged_service`,
because no list an engine is asked for says whether a service runs with `--privileged` — only
`inspect` does, and `inspect` also prints the environment, which this agent never asks for; and
`container.image_unknown`, because every image a container of an engine runs is an image that
engine lists, so an image it does not know is only the race between two commands of the dump.

**Silence.** The first reading is the baseline, as everywhere. On top of that,
`rules/did_not_answer.rs` claims every row of a subject its engine did not answer for on one side
of the comparison: a command that failed is not everything it held removed, and the round it
answers again is not everything new. An engine installed or uninstalled between two readings is
learned the same way. What arrived while the engine was silent is learned as a baseline.

**Hidden values** are listed in `Finding::redacted`: a label whose value was hidden as
`/after/labels/<name>` (with `~` and `/` escaped), and a secret's value, which is never read, as
`/after/value`.

## The screen

`views/` declares a `Section` named `containers`, which the console merges with the `/proc`
section of `vigil-containers` into one screen of three groups: `host · docker · podman`. Each
engine has a list per subject, one pane type `Of(engine, list)`: `containers`, `images`,
`volumes`, `networks`, `compose`, `registries`, and for podman `pods` and `secrets`. Docker's six
names fit the second row in eighty columns; podman's eight fold into `< images >` there.

Every list offers its columns (fewer at eighty columns, all of them from 118), search, a sort by
any column, the detail of a row with every field and what it means, a footer and `S`: every row
of these lists is something a rule reports on, and the detail carries the finding key the rules
raise, which a test compares with what the rules produce. Containers, volumes and networks narrow
by compose project (`f`). A row with a dangerous setting is listed first.

The engine's own facts — version, storage driver, rootless or not — are in the footer of every
list of that engine, second after the count. A list whose engine is not installed, not named in
`engines:`, or whose dump could not be read is hidden, and the group says why in one line; a
subject the engine did not answer for says so instead of claiming the engine holds none.

## Configuration

The block is named after the module, like every other one: `containers-engines`, in the
collectors' files (`/etc/vigil/collectors/containers.yaml`, as the second document).

```yaml
containers-engines:
  schedule: 120
  engines: [docker, podman]
  dump_seconds: 120
  report:
    images: true
    volumes: true
    networks: true
    projects: true
    pods: true
    secrets: true
```

The switches under `report:` turn off what comes and goes for one subject each — for a host whose
images change on every deploy. The dangerous settings are not switchable.

Declared through `settings_key()` and checked at start-up: an engine this build does not read,
an engine named twice, a `dump_seconds` of zero and a key nobody declares are all refused by
name before the daemon starts. `dump_seconds` is the period `vigil-containers.timer` runs at;
change one and change the other, or the agent calls a current reading stale.

A `vigil.yaml` of the former layout, which holds its collectors itself, still configures the
engines under a top-level `containers:` block: the daemon reads that key as `containers-engines`.

## The fixture

`fixture::engines()` is the contract with the rules and the screens that come after this crate.
It is built by the real parsers from two dump documents written the way docker and podman
really print (`fixture::docker`, `fixture::podman`), and it holds 31 rows:

- **docker**, present, version 27.1.1: three images — `nginx:1.27-alpine` with a digest,
  `shop/api:2026.09.1` without one, and one untagged; two volumes, one of them anonymous;
  three networks (`bridge`, `host`, `shop_default`); three containers in two compose projects,
  `shop` (web and api, one of them mounting `/etc` and publishing 443) and `metrics` (an agent
  on the **host network**, running from the **untagged** image); an **insecure registry**
  `registry.local:5000` and a mirror over https, from `daemon.json`; one container labelled
  with a token, hidden;
- **podman**, present, rootless, version 5.1.1: two images, one of them untagged; a volume
  `etc_backup` **bound to `/etc`**; two bridge networks, `podman1` **with the subnet
  10.89.0.0/24** and internal; two containers in the **pod `tools`**, one publishing
  127.0.0.1:2222 and labelled with a vault token, hidden; one **secret**
  `shop-database-password`, name and time only; the same insecure registry plus two search
  registries, from `registries.conf`.

`fixture::only_docker()` is the same host with podman uninstalled. `fixture::row()` and the
named constants (`UNTAGGED_IMAGE`, `HOST_NETWORK_CONTAINER`, `VOLUME_FROM_ETC`,
`BRIDGE_WITH_A_SUBNET`, `INSECURE_REGISTRY`, `POD`, `SECRET`, `PROJECTS`) reach a row by name.

## Cost

`cargo run --release --example engines -p vigil-engines` reads two dumps and builds the
snapshot. On a fixture-sized host (31 rows, 7.9 KB of engine output) the projection is about
0.12 ms; at 200 copies of it (624 rows, 1.6 MB) it is about 9.4 ms, so it grows with the
output and not worse. The daemon reads it every 120 seconds.

The same example judges one round with the diff included. On the fixture a round in which
nothing moved costs 0.004 ms, a round in which a seventh of the rows went or changed 0.022 ms,
and a round in which every row arrived at once (25 findings) 0.045 ms. At 624 rows the three are
0.17 ms, 0.42 ms and 1.9 ms (817 findings). It then walks every list at 624 rows: the largest,
docker's 600 containers, builds its index in 0.66 ms and its footer counts in 0.10 ms, answers a
keystroke from the index in 0.011 ms and draws a window of forty rows in 0.027 ms; docker's
images count the containers that run each image, which costs 0.18 ms per window of rows. On
the console, a key press and a redraw of the containers screen at 120×40 went from 0.23 ms
before the engines had lists to 0.25–0.26 ms, on the host list and on docker's containers alike.

## Dependencies

`vigil-model` (`Snapshot`), `vigil-collect` (the `Collector` port, `Health`, `CollectError`),
`vigil-rules` (`RuleSet`, the ports, the list of paths of this host), `vigil-view` (the screen
as data), `vigil-module` (`Module`, `Settings`), `serde`/`serde_json`. It starts no program:
everything it touches is a file.

## Context

```
vigil-containers.timer ──► vigil-container-dump ──► /var/lib/vigil/containers/*.json
                                    │                          │
                          (links this crate for                │
                           the engine vocabulary)              ▼
                                    └──────────────►  vigil-engines ──► bin/vigild
                            /etc/docker/daemon.json ──►    │
                       /etc/containers/registries.conf ►   └──► bin/vigil (the docker and
                                                                podman groups of the
                                                                containers screen)
```
