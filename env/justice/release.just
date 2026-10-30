set shell := ["bash", "-c"]
set working-directory := '../..'

import 'common.just'

[private]
default:
    @{{ JUST }} --list release --list-heading $'just release <what>\n' --list-prefix '    '

# The version this working copy would release
version:
    ./env/scripts/release.sh version

# Check a tag against the version in Cargo.toml before pushing it
check TAG:
    ./env/scripts/release.sh check-tag {{ TAG }}

# Build the packages for TARGET and name them the way the release does
assets:
    {{ JUST }} package all
    ./env/scripts/release.sh collect
    ./env/scripts/release.sh checksums

# Build the macOS packages and name them the way the release does
assets-macos:
    {{ JUST }} package macos
    TARGET=universal-apple-darwin ./env/scripts/release.sh collect
    ./env/scripts/release.sh checksums

# The notes the release would carry, read out of CHANGELOG.md
notes:
    ./env/scripts/release.sh notes

# CHANGELOG.md anew from the commits between v* tags
changelog:
    ./env/scripts/changelog.sh rebuild

# The version the next release takes (auto from the commits, or patch/minor/major)
next level='auto':
    @./env/scripts/changelog.sh next {{ level }}

# Bump the version in Cargo.toml unless it is already past the last tag, then write its CHANGELOG.md section
bump level='auto':
    ./env/scripts/changelog.sh release {{ level }}
