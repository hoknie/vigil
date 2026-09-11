printf '\n  vigil workbench — Alpine, musl, %s\n' "$(rustc --version 2>/dev/null)"

cat <<'BANNER'
  The working copy is at /work (mounted); build output goes to /build (a volume).

    vigil ui                  the console
    vigild <config.yaml>      the daemon
    just check                fmt, clippy and the tests
    ./env/docker/smoke.sh     build, test, start, report

    cat /var/log/vigil/findings.ndjson   what it has reported

  Try it: run "nc -l -p 4444 &" here, wait for the next reading, and watch a finding appear.

BANNER
