CARGO := env('CARGO', 'cargo')
CONFIG := env('CONFIG', '')

COMPOSE := env('COMPOSE', 'docker compose -f env/docker/compose.yaml')
RUN := COMPOSE + ' run --rm --build vigil'

JUST := just_executable() + ' --justfile ' + quote(justfile())

TARGET := env('TARGET', 'x86_64-unknown-linux-musl')
PLATFORM := env('PLATFORM', 'linux/amd64')
CLEAN_DEB := env('CLEAN_DEB', 'debian:12')
CLEAN_RPM := env('CLEAN_RPM', 'almalinux:9')

NEXT_VERSION := env('NEXT_VERSION', '0.1.1')

CLEAN := 'docker run --rm --platform ' + PLATFORM + ' -v "$PWD":/work -w /work'
