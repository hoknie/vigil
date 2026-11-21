# homebrew-vigil

> The Homebrew tap of [vigil](https://github.com/hoknie/vigil), a host protection agent.

This repository holds one formula, `Formula/vigil.rb`. It is written by the release of `vigil`
and is not edited by hand.

## Install

    brew install hoknie/vigil/vigil

The formula installs the universal macOS build of the release, for macOS 11 and later, on Apple
silicon and Intel.

## After installing

The agent runs as your own account, with no `sudo`:

    vigild configure
    brew services start vigil
    vigil ui

Without root the agent cannot see everything a Mac holds, and each collector says what it could
not read. An agent run by root is installed from the `.pkg` of the release.

## Upgrade

    brew upgrade vigil
    brew services restart vigil

Your configuration under `~/.config/vigil` and the state under `~/.local/state/vigil` are not
touched.

## Remove

    brew services stop vigil
    brew uninstall vigil

The configuration, the state and the logs stay in your home directory. Delete `~/.config/vigil`,
`~/.local/state/vigil` and `~/Library/Logs/vigil` to remove them.
