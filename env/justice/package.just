TARGET := env('TARGET', 'x86_64-unknown-linux-musl')
PLATFORM := env('PLATFORM', 'linux/amd64')
CLEAN_DEB := env('CLEAN_DEB', 'debian:12')
CLEAN_RPM := env('CLEAN_RPM', 'almalinux:9')

NEXT_VERSION := env('NEXT_VERSION', '0.1.1')

CLEAN := 'docker run --rm --platform ' + PLATFORM + ' -v "$PWD":/work -w /work'

# The three statically linked musl binaries the packages wrap
[group('package')]
package-binaries:
    {{ COMPOSE }} run --rm --build -T -e TARGET={{ TARGET }} vigil ./env/scripts/package.sh binaries

# Build the .deb into ./dist
[group('package')]
package-deb: package-binaries
    {{ CLEAN }} {{ CLEAN_DEB }} timeout -s KILL 300 ./env/scripts/package.sh deb

# Build the .rpm into ./dist
[group('package')]
package-rpm: package-binaries
    {{ CLEAN }} {{ CLEAN_RPM }} bash -c 'timeout -s KILL 600 sh -c "dnf -y -q install rpm-build >/dev/null && ./env/scripts/package.sh rpm"'

# Build packages: deb + rpm
[group('package')]
package: package-deb package-rpm

# Install all packages on clean machines and check files, upgrade and removal
[group('package')]
package-verify: package
    {{ CLEAN }} {{ CLEAN_DEB }} timeout -s KILL 300 env VERSION={{ NEXT_VERSION }} ./env/scripts/package.sh deb
    {{ CLEAN }} {{ CLEAN_RPM }} bash -c 'timeout -s KILL 900 sh -c "dnf -y -q install rpm-build >/dev/null && VERSION={{ NEXT_VERSION }} ./env/scripts/package.sh rpm"'
    {{ CLEAN }} {{ CLEAN_DEB }} timeout -s KILL 300 env NEXT={{ NEXT_VERSION }} ./env/scripts/package-check.sh deb
    {{ CLEAN }} {{ CLEAN_RPM }} timeout -s KILL 300 env NEXT={{ NEXT_VERSION }} ./env/scripts/package-check.sh rpm

# Start the shipped unit under a systemd and measure what its hardening allows
[group('package')]
package-systemd: package-deb
    docker build --platform {{ PLATFORM }} --tag vigil-systemd-check:deb12 --file env/docker/systemd-check.Dockerfile env/docker
    -docker rm --force vigil-systemd-check
    docker run --detach --name vigil-systemd-check --platform {{ PLATFORM }} \
        --privileged --cgroupns host --volume /sys/fs/cgroup:/sys/fs/cgroup:rw \
        --tmpfs /run --tmpfs /run/lock --volume "$PWD":/work --workdir /work \
        vigil-systemd-check:deb12
    trap 'docker rm --force vigil-systemd-check >/dev/null' EXIT INT TERM; \
        docker exec vigil-systemd-check timeout -s KILL 900 /work/env/scripts/package-systemd-check.sh

# `systemd-analyze verify` over every unit the package ships, as installed
[group('package')]
package-unit: package-deb
    {{ CLEAN }} {{ CLEAN_DEB }} bash -c 'timeout -s KILL 600 sh -c "apt-get -qq update && apt-get -qq install -y --no-install-recommends systemd nftables >/dev/null && dpkg -i \"\$(ls -t dist/vigil_*_amd64.deb | head -1)\" >/dev/null && systemd-analyze verify /usr/lib/systemd/system/vigild.service /usr/lib/systemd/system/vigil-firewall.service /usr/lib/systemd/system/vigil-firewall.timer && echo \"the units verify\""'
