# vigil-view

The vocabulary a module uses to say what its screen holds, and the only thing the console
knows how to draw.

## The decision it follows from

A subject watched by this agent is one crate — reading, parsers, rules and screen together
(`docs/designs/2026-09-12-DESIGN-modules.md`). A module must therefore be able to describe a
screen without drawing one: if it drew, it would link `ratatui`, and `ratatui` would arrive in
the daemon's dependency graph with it, because Cargo features are additive across a build and
a feature flag cannot hold that line.

So a module answers with data — columns, row keys, cells, pieces of a detail, a notice, a
tally — and `bin/vigil` turns that into a terminal. The skeleton it is drawn into is one for
every section: a row of names, a sentence about the pane, a filter, a search, a table, a
footer.

## What is here

- `ports/` — `Section` (a section of the console) and `Pane` (one position of its menu);
- `types/` — what those two answer with: `Column`/`Width`/`Room` for the table's shape,
  `RowKey` and `Cell` for its contents, `Piece` for the detail under a row, `Notice` for an
  empty or unavailable screen, `Toggle`/`Showing`/`Sorting`/`Offers` for what the reader has
  narrowed it to;
- `conformance/` — the suite every pane passes: cells match columns at both widths, the keys
  it offers are the reading's own, no key twice, a detail that says something.

`rows()` and `cells()` are separate on purpose: a reading can hold twenty thousand rows, the
terminal shows forty. Keys are cheap to build for all of them; cells are built for what is on
screen.

## Dependencies

`vigil-model` — nothing else. No io, no async, no `ratatui`, and no crate that has one.

## Context

```
vigil-model ──► vigil-view ──► collectors/vigil-<subject>/views/ ──► bin/vigil (draws it)
```
