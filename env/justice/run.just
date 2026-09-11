# Run the daemon against CONFIG (defaults to the shipped example)
[group('run')]
daemon:
    {{ CARGO }} run --bin vigild -- {{ CONFIG }}

# Run the terminal console (q or Esc to leave)
[group('run')]
console:
    {{ CARGO }} run --bin vigil -- ui

# Print one screen as text (screen=summary by default)
[group('run')]
capture screen='summary':
    {{ CARGO }} run --bin vigil -- capture --screen {{ screen }}
