set shell := ["bash", "-c"]
set working-directory := '../..'

import 'common.just'

[private]
default:
    @{{ JUST }} --list package-check --list-heading $'just package-check <what>\n' --list-prefix '    '

# Install all packages on clean machines and check files, upgrade and removal
verify:
    {{ JUST }} package all
    {{ CLEAN }} {{ CLEAN_DEB }} timeout -s KILL 300 env VERSION={{ NEXT_VERSION }} ./env/scripts/package.sh deb
    {{ CLEAN }} {{ CLEAN_RPM }} bash -c 'timeout -s KILL 900 sh -c "dnf -y -q install rpm-build >/dev/null && VERSION={{ NEXT_VERSION }} ./env/scripts/package.sh rpm"'
    {{ CLEAN }} {{ CLEAN_DEB }} timeout -s KILL 300 env NEXT={{ NEXT_VERSION }} ./env/scripts/package-check.sh deb
    {{ CLEAN }} {{ CLEAN_RPM }} timeout -s KILL 300 env NEXT={{ NEXT_VERSION }} ./env/scripts/package-check.sh rpm

# Start the shipped unit under a systemd and measure what its hardening allows
systemd:
    {{ JUST }} package deb
    docker build --platform {{ PLATFORM }} --tag vigil-systemd-check:deb12 --file env/docker/systemd-check.Dockerfile env/docker
    -docker rm --force vigil-systemd-check
    docker run --detach --name vigil-systemd-check --platform {{ PLATFORM }} \
        --privileged --cgroupns host --volume /sys/fs/cgroup:/sys/fs/cgroup:rw \
        --tmpfs /run --tmpfs /run/lock --volume "$PWD":/work --workdir /work \
        vigil-systemd-check:deb12
    trap 'docker rm --force vigil-systemd-check >/dev/null' EXIT INT TERM; \
        docker exec vigil-systemd-check timeout -s KILL 900 /work/env/scripts/package-systemd-check.sh

# `systemd-analyze verify` over every unit the package ships, as installed
unit:
    {{ JUST }} package deb
    {{ CLEAN }} {{ CLEAN_DEB }} bash -c 'timeout -s KILL 600 sh -c "apt-get -qq update && apt-get -qq install -y --no-install-recommends systemd nftables >/dev/null && dpkg -i \"\$(ls -t dist/vigil_*_amd64.deb | head -1)\" >/dev/null && systemd-analyze verify /usr/lib/systemd/system/vigild.service /usr/lib/systemd/system/vigil-firewall.service /usr/lib/systemd/system/vigil-firewall.timer /usr/lib/systemd/system/vigil-containers.service /usr/lib/systemd/system/vigil-containers.timer && echo \"the units verify\""'

# Look inside the macOS .pkg, and run its install scripts against a scratch volume
macos:
    ./env/scripts/package-macos.sh check

# Install the agent on eleven Linux distributions in containers and print what each collector reads there
distributions *images:
    ./env/scripts/distributions.sh build
    ./env/scripts/distributions.sh run {{ images }}
