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

# CHANGELOG.md anew from the commits between v* tags
[group('release')]
changelog:
    ./env/scripts/changelog.sh rebuild

# The version the next release takes (auto from the commits, or patch/minor/major)
[group('release')]
next-version level='auto':
    @./env/scripts/changelog.sh next {{ level }}

# Bump the version in Cargo.toml unless it is already past the last tag, then write its CHANGELOG.md section
[group('release')]
release-bump level='auto':
    ./env/scripts/changelog.sh release {{ level }}
