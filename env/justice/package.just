set shell := ["bash", "-c"]
set working-directory := '../..'

import 'common.just'

[private]
default:
    @{{ JUST }} --list package --list-heading $'just package <what>\n' --list-prefix '    '

# Build packages: deb + rpm
all: deb rpm

# The four statically linked musl binaries the packages wrap
binaries:
    {{ COMPOSE }} run --rm --build -T -e TARGET={{ TARGET }} vigil ./env/scripts/package.sh binaries

# Build the .deb into ./dist
deb: binaries
    {{ CLEAN }} {{ CLEAN_DEB }} timeout -s KILL 300 ./env/scripts/package.sh deb

# Build the .rpm into ./dist
rpm: binaries
    {{ CLEAN }} {{ CLEAN_RPM }} bash -c 'timeout -s KILL 600 sh -c "dnf -y -q install rpm-build >/dev/null && ./env/scripts/package.sh rpm"'

# Build the macOS .pkg and .tar.gz into ./dist from universal binaries, and check them
macos:
    ./env/scripts/package-macos.sh all
