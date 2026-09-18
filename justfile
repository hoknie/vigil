set shell := ["bash", "-c"]

CARGO := env('CARGO', 'cargo')
CONFIG := env('CONFIG', '')

COMPOSE := env('COMPOSE', 'docker compose -f env/docker/compose.yaml')
RUN := COMPOSE + ' run --rm --build vigil'

[private]
default:
    @{{ just_executable() }} --list

import 'env/justice/build.just'
import 'env/justice/run.just'
import 'env/justice/quality.just'
import 'env/justice/docker.just'
import 'env/justice/package.just'
import 'env/justice/release.just'
import 'env/justice/docs.just'
import 'env/justice/housekeeping.just'
