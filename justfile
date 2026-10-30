set shell := ["bash", "-c"]

import 'env/justice/common.just'
import 'env/justice/quality.just'

[private]
default:
    @{{ just_executable() }} --list

# Build, docs and clean: just build <what>
[group('build')]
mod build 'env/justice/build.just'

# The daemon and the console on this machine: just run <what>
[group('run')]
mod run 'env/justice/run.just'

# Formatting and lint: just qa-lint <what>
[group('quality')]
mod qa-lint 'env/justice/qa-lint.just'

# Tests, samples, coverage and cost: just qa-test <what>
[group('quality')]
mod qa-test 'env/justice/qa-test.just'

# The workbench container: just docker-env <what>
[group('docker')]
mod docker-env 'env/justice/docker-env.just'

# The gates and checks that run in containers: just docker-gates <what>
[group('docker')]
mod docker-gates 'env/justice/docker-gates.just'

# Build the packages: just package <what>
[group('package')]
mod package 'env/justice/package.just'

# Install and check the packages on clean machines: just package-check <what>
[group('package')]
mod package-check 'env/justice/package-check.just'

# Version, changelog, notes and assets of a release: just release <what>
[group('release')]
mod release 'env/justice/release.just'
