# vigil on macOS

`vigil` watches the Mac it runs on — what listens, who can log in, what runs, what starts by
itself — compares each reading with the last one and turns the difference into findings.
`vigild` is the daemon, `vigil` the console.

## What is installed where

| what | where |
|---|---|
| the daemon | `/usr/local/sbin/vigild`, run by launchd as `vigil.vigild` (`/Library/LaunchDaemons/vigil.vigild.plist`) |
| the console | `/usr/local/bin/vigil` |
| helper programs | `/usr/local/libexec/vigil/` |
| configuration | `/usr/local/etc/vigil/vigil.yaml`, `collectors/`, `suppressions/`, `reporters/`, `watch_fs.yaml` |
| the defaults this version ships | `/usr/local/share/vigil/defaults/` |
| baselines, findings journal | `/usr/local/var/lib/vigil` (0700) |
| what the daemon says about itself | `/usr/local/var/log/vigil/vigild.log` |
| the console socket | `/var/run/vigil/vigil.sock` (0600, in a 0700 directory the daemon makes at every start) |
| readings run by launchd jobs of their own | `vigil.firewall`, `vigil.containers`, `vigil.launches` in `/Library/LaunchDaemons` |

## Installing the .pkg

The package is not signed with an Apple Developer ID and not notarized, so Gatekeeper refuses
to open it with a double click. Either:

    sudo installer -pkg vigil_<version>.macos.universal.pkg -target /

or, in Finder, Control-click the package, choose Open, and confirm. On macOS 15 and later, open
it once, then allow it under System Settings → Privacy & Security → "Open Anyway".

Check what you downloaded against `SHA256SUMS` of the same release before you do either:

    shasum -a 256 -c SHA256SUMS --ignore-missing

Signing and notarization need the owner's Developer ID certificate; until then the checksum is
what ties the file to the release.

## Upgrades keep your configuration

A file under `/usr/local/etc/vigil` that you changed is never written over. The defaults of the
new version are put beside it as `<file>.new`, and the installer says which files those are. A
file you never changed is brought to the new defaults.

## After installing

Nothing has to be typed: the package starts `vigild`, and the firewall and container-engine
readings with it. Open the console with:

    sudo vigil ui

The shipped configuration watches everything this Mac can be read for. To see what fits it and
write that instead:

    vigild configure --dry-run          what this Mac can be watched with, and the files for it
    sudo vigild configure --force       write them; what was there is kept as .previous
    sudo launchctl kickstart -k system/vigil.vigild

The daemon reads the files of every account only with **Full Disk Access**: add
`/usr/local/sbin/vigild` under System Settings → Privacy & Security → Full Disk Access. Without
it, a collector says what it could not read instead of reporting an empty Mac.

Program launches are read from `eslogger` (macOS 13 and later), which needs root and Full Disk
Access for the program that runs it. Without them the `launches` collector is unavailable and
says so.

## What the console may do on a Mac

- **close a socket or stop a program** (`killing.from_the_console`): stopping a program works as
  on Linux. Closing a socket and leaving its process running is refused: macOS has no
  `SOCK_DESTROY`.
- **change accounts** (`accounts.from_the_console`): refused on macOS, with the reason, and the
  refusal is a finding. The accounts of a Mac live in Directory Services.
- **stop and start what the Mac starts by itself** (`units.from_the_console`): a launchd job is
  refused on macOS, with the reason, and the refusal is a finding; a line of a crontab is
  commented out and back in as on Linux.

## Removing it

    sudo /usr/local/libexec/vigil/uninstall            programs and jobs; configuration, findings and logs are kept
    sudo /usr/local/libexec/vigil/uninstall --purge    all of it

## From the .tar.gz instead

The archive holds the same universal binaries, the launchd jobs in `launchd/`, the default
configuration in `config/` (already with the paths of a Mac) and `uninstall`:

    sudo install -d -m 0755 /usr/local/sbin /usr/local/bin /usr/local/libexec/vigil
    sudo install -m 0755 vigild /usr/local/sbin/vigild
    sudo install -m 0755 vigil /usr/local/bin/vigil
    sudo install -m 0755 vigil-container-dump vigil-firewall-dump vigil-launches-spool uninstall /usr/local/libexec/vigil/
    sudo install -d -m 0700 /usr/local/etc/vigil /usr/local/etc/vigil/collectors \
        /usr/local/etc/vigil/suppressions /usr/local/etc/vigil/reporters \
        /usr/local/var/lib/vigil /usr/local/var/log/vigil
    sudo cp -n config/vigil.yaml config/watch_fs.yaml /usr/local/etc/vigil/
    sudo cp -n config/collectors/*.yaml /usr/local/etc/vigil/collectors/
    sudo chmod 0600 /usr/local/etc/vigil/*.yaml /usr/local/etc/vigil/collectors/*.yaml
    sudo install -m 0644 launchd/vigil.vigild.plist /Library/LaunchDaemons/
    sudo launchctl bootstrap system /Library/LaunchDaemons/vigil.vigild.plist

A binary downloaded by a browser carries the quarantine attribute; `xattr -d
com.apple.quarantine <file>` takes it off once you have checked the checksum.

## With Homebrew

    brew install hoknie/vigil/vigil
    vigild configure
    brew services start vigil
    vigil ui

The formula installs the universal build of the release for macOS 11 and later, and the agent
it starts is the one described under "Without sudo": it runs as your account, with the same
places. Homebrew puts the programs in its own prefix instead (`vigild` in `sbin`, `vigil` in `bin`,
the helper programs and `uninstall` in `libexec`, the defaults in `share/vigil/defaults`), and
`vigild configure` writes no launchd job, because the service is `brew services`.

If an earlier install from the archive left `~/Library/LaunchAgents/vigil.vigild.plist`, `vigild
configure` names it and leaves it alone: it would start a second agent on the same socket, so
boot it out and delete it first.

Upgrade with `brew upgrade vigil`, then `brew services restart vigil`. Remove with
`brew services stop vigil` and `brew uninstall vigil`; the configuration, the state and the logs
in your home directory stay.

An agent run by root is not a Homebrew service: install the `.pkg` for it.

## Without sudo

An ordinary account can run its own agent, with no package and no `sudo`. Unpack the `.tar.gz` of the
release, put `vigild` and `vigil` on your `PATH` (if Gatekeeper refuses them, run
`xattr -d com.apple.quarantine vigild vigil`), then:

    vigild configure
    launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/vigil.vigild.plist
    vigil ui

Instead of the launchd job, `vigild ~/.config/vigil/vigil.yaml` runs the agent in a terminal.

| what | where |
|---|---|
| configuration | `~/.config/vigil/vigil.yaml`, `collectors/`, `suppressions/`, `reporters/`, `watch_fs.yaml` |
| baselines, findings journal | `~/.local/state/vigil` |
| the console socket | `~/.local/state/vigil/vigil.sock` |
| what the daemon says about itself | `~/Library/Logs/vigil/vigild.log` |
| the launchd job | `~/Library/LaunchAgents/vigil.vigild.plist`, in the `gui/<uid>` domain |

Without root the agent does not see the processes and sockets of other accounts, the crontab of
other accounts, the firewall, program launches or the container engines. Each collector that sees
less says so itself, in the console and as `agent.collector.degraded`.

An account's agent and the system agent can live on one Mac; the console opens the account's socket
when it exists and the system one otherwise (`vigil ui --socket PATH` names one).

An agent that was run without root before with a configuration that writes no `state_dir` kept its
state under `/usr/local/var/lib/vigil`; it now starts a new baseline under `~/.local/state/vigil`.
Write `state_dir: /usr/local/var/lib/vigil` in its configuration to keep the old one.
