# The version this working copy would release
[group('release')]
release-version:
    ./env/scripts/release.sh version

# Check a tag against the version in Cargo.toml before pushing it
[group('release')]
release-check TAG:
    ./env/scripts/release.sh check-tag {{ TAG }}

# Build the packages for TARGET and name them the way the release does
[group('release')]
release-assets: package
    ./env/scripts/release.sh collect
    ./env/scripts/release.sh checksums

# Build the macOS packages and name them the way the release does
[group('release')]
release-assets-macos: package-macos
    TARGET=universal-apple-darwin ./env/scripts/release.sh collect
    ./env/scripts/release.sh checksums

# The notes the release would carry, read out of CHANGELOG.md
[group('release')]
release-notes:
    ./env/scripts/release.sh notes
