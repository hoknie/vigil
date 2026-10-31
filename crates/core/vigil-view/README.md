# vigil-view

> A subject's screen, described as data.

## Why

A subject crate describes its screen — rows, columns, the card of a row, what a person may do with
it — in the types of this crate. The console turns that description into a terminal. `ratatui`
stays in `vigil` alone, and a screen is tested by comparing values.

## What's inside

| | |
|---|---|
| **`Section`, `Pane`** | the ports a subject implements: a section on the main screen, panes on its own screen |
| **Rows** | `Rows`, `Column`, `Cell`, `Width`, `Sorting`, `Showing`, `Index` |
| **Cards** | `Piece` and `Emphasis` for the text of a card |
| **Actions** | `Offers` for what the console may do with a row; `Form` and `Field` for what it asks first |
| **helpers** | times of day, sizes in bytes, base names, the text a search looks through |
| **`conformance`** | the checks every subject's section passes |

## Depends on

- `vigil-model`
- `serde_json`, `memchr`

## Where it fits

```text
vigil-model ──► vigil-view ──► collectors/vigil-<subject>/views
                    │
                    └────────► vigil · draws it with ratatui
```
