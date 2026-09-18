# Run the daemon against CONFIG, or against config/ copied to target/vigil-config
[group('run')]
daemon:
    {{ CARGO }} run --bin vigild -- {{ if CONFIG == "" { "\"$(./env/scripts/config-here.sh)\"" } else { CONFIG } }}

# Run the terminal console (q or Esc to leave)
[group('run')]
console:
    {{ CARGO }} run --bin vigil -- ui

# Print one screen as text (screen=summary by default)
[group('run')]
capture screen='summary':
    {{ CARGO }} run --bin vigil -- capture --screen {{ screen }}
