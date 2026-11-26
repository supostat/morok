# The guide of the kit

MOROK is a UI kit for macOS that draws the whole interface itself, through Metal: no `AppKit`
controls and no browser. An application describes its windows as trees of elements, styles them
with a subset of CSS compiled at build time, and changes them by writing signals.

This guide is for whoever writes an application on the kit. It lives in `crates/morok/docs` and
is attached to the documentation of the crate as the module `morok::guide`; every Rust example in
it is compiled by `cargo test -p morok --doc`. Paths in backticks are given from the root of the
repository of the kit.

## What the kit needs

- macOS 26 or later on Apple Silicon: the renderer is written for Metal 4.
- Rust 1.94 or later, edition 2024.
- The main thread: an application is created and run on it.

## The layers

An application depends on one crate, `morok`. The layers of the kit are its modules.

| Module | What it holds |
|---|---|
| `morok` | `Application`, `Window`, `Styles`, `WindowOptions` and what the application gives the platform: appearances, canvases, pictures. |
| `morok::components` | The components: `Button`, `TextField`, `Table`, `Popover` and the rest. |
| `morok::components::view` | What components are built with: `Mount`, `ElementBuilder`, `Property`, `ElementEvent`. |
| `morok::reactive` | The signal graph: `Signal`, `Memo`, `Effect`, `Runtime`. |
| `morok::core` | The element tree and its vocabulary: `ElementId`, keys and pointer events, geometry, drags. |
| `morok::style` | The compiled form of styles: `Class`, `Token`, `Declaration`, `Theme`, `States`. |

One more crate works at build time: `morok-css` compiles the tokens and the stylesheets of an
application into Rust tables. An application that is content with the look of the kit needs no
build script: `Styles::of_kit()` gives it the tables of the kit.

## How it works, in one paragraph

A component is a function that is called once. It is handed a `Mount`, creates its signals,
builds its elements and returns the outermost of them. A property of an element is given either a
value, which stays, or a signal, which the property follows from then on. A handler of an event
writes signals; before the next frame the kit runs the effects those writes woke, and exactly the
elements bound to the changed signals change. No component runs twice.

## How to find a thing

- A name: [the index](index.md) lists every name the facade, the components and the view export,
  with what it is for, the page of the catalog that shows it and its source file.
- A working example: the catalog, `crates/catalog`, is an application built on the kit — one page
  for every family of components in `crates/catalog/src/pages`, and two whole screens,
  `crates/catalog/src/file_manager.rs` and `crates/catalog/src/disk_map.rs`.
  `cargo run -p morok-catalog` shows it.
- What a method does and when it panics: the doc comment of the item. Every public item of the kit
  has one; `cargo doc -p morok --open` renders them, this guide among them.
- What CSS a stylesheet may hold: `crates/css/docs/css-subset.md`, rendered from the tables of the
  compiler.

## The chapters

1. [An application](application.md) — from an empty crate to a window; styles of its own; themes;
   several windows; `defer`.
2. [Elements](elements.md) — `Mount` and `ElementBuilder`: elements, text, classes, children,
   properties bound to signals, handlers.
3. [Signals](signals.md) — signals, memos and effects, and reaching them from outside a component.
4. [Components](components.md) — how a component is built and mounted, family by family.
5. [Styles](styles.md) — tokens, classes, states, selectors, units, `var()`, `calc()`,
   `@keyframes`, and what the compiler refuses.
6. [Overlays](overlays.md) — popovers, menus, dialogs, sheets, toasts, and who owns them.
7. [Pictures](pictures.md) — images, icons and canvases.
8. [Rules](rules.md) — what panics, and what a caller must keep.
9. [The index](index.md) — every exported name.
