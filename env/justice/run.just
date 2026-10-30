set shell := ["bash", "-c"]
set working-directory := '../..'

import 'common.just'

[private]
default:
    @{{ JUST }} --list run --list-heading $'just run <what>\n' --list-prefix '    '

# Run the daemon against CONFIG, or against config/ copied to target/vigil-config
daemon:
    {{ CARGO }} run --bin vigild -- {{ if CONFIG == "" { "\"$(./env/scripts/config-here.sh)\"" } else { CONFIG } }}

# Run the terminal console (q or Esc to leave)
console:
    {{ CARGO }} run --bin vigil -- ui

# Print one screen as text (screen=summary by default)
capture screen='summary':
    {{ CARGO }} run --bin vigil -- capture --screen {{ screen }}
