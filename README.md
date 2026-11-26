# MOROK

A UI kit for macOS that draws the whole interface itself, through Metal: no `AppKit` controls and
no browser. An application describes its windows as trees of elements, styles them with a subset
of CSS compiled at build time, and changes them by writing signals. The components — buttons,
fields, tables, menus, dialogs — come with the kit, with keyboard and accessibility support.

## What it needs

- macOS 26 or later on Apple Silicon: the renderer is written for Metal 4.
- Rust 1.94 or later.

## Writing an application on the kit

**Start with the guide: [crates/morok/docs/overview.md](crates/morok/docs/overview.md).** It
leads to every chapter — an application from an empty crate, elements, signals, components,
styles, overlays, pictures, the rules a caller must keep — and to an index of every exported
name. Every Rust example in the guide is compiled by the tests of the kit.

An application depends on one crate, the facade:

```toml
[dependencies]
morok = { git = "https://github.com/supostat/morok" }
```

Three more places answer what the guide leaves out:

- The doc comment of every public item: `cargo doc -p morok --open`.
- The CSS a stylesheet may hold: [crates/css/docs/css-subset.md](crates/css/docs/css-subset.md).
- A whole application to read: the catalog, in `crates/catalog`.

## The catalog

The catalog shows every family of components on a page of its own, in both color schemes, and two
screens built from them — a file manager and a disk map.

```text
cargo run -p morok-catalog
```

## Working on the kit

`docs/MOROK-ARCHITECTURE.md` describes how the kit is built. One command runs what a change must
pass — formatting, clippy, the comment rules and the checks that hold the documentation to the
code:

```text
cargo xtask check
```

## License

MIT or Apache-2.0, at your option: `LICENSE-MIT`, `LICENSE-APACHE`.
