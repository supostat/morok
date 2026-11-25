# MOROK — Architecture

*Morok* (Russian: морок) is a glamour, an illusion — you see what is not there. The interface looks
like CSS from a browser, yet there is no browser — only Rust, Core Text and Metal. This document
describes how the kit is built; decisions and their rationale are in `docs/GRILL-morok.md`, the
visual language and the catalog in `design/morok-design-book.html`, prior art in §14 (Facet).

## 1. Layers

```
┌──────────────────────────────────────────────────────────────────────┐
│  Application: signal-based components                                  │
│  fn(props) runs once; signals → targeted node updates                  │
├──────────────────────────────────────────────────────────────────────┤
│  Kit components: Button, Table, TextField, Popover …                   │
│  behaviour + states + .css (styles) + AccessKit role                   │
├──────────────────────────────────────────────────────────────────────┤
│  Element tree (retained, arena + IDs)                                  │
│  style (compiled) · layout (Taffy) · damage · focus · a11y             │
├──────────────────────────────────────────────────────────────────────┤
│  Display list → instances (rect / glyph / icon / image / canvas)       │
├──────────────────────────────────────────────────────────────────────┤
│  Platform (objc2): NSWindow/NSView, CAMetalLayer, Metal 4, Core Text,  │
│  NSTextInputClient, NSMenu, NSVisualEffectView, AccessKit adapter      │
└──────────────────────────────────────────────────────────────────────┘
```

The build pipeline sits alongside: the app's `tokens.css` and the components' `.css` are compiled
at build time into static tables (token constants for the light and dark themes, rules per class and
state). CSS is never parsed at runtime.

## 2. Crate layout

The kit is a workspace of crates under `crates/`, each depending only on the ones named. The
layering is held by the compiler: a crate cannot reach a layer it does not depend on.

| Crate | Package | Holds | Depends on |
|---|---|---|---|
| `crates/style` | `morok-style` | The schema of tokens and compiled styles: `Token<V>`, `Theme`, `StyleSheet`, `Class`, declarations, states, keyframes. | — |
| `crates/css` | `morok-css` | The build-time compiler: tokens files and stylesheets into the tables of `morok-style`, diagnostics, the `build.rs` entry points. | `morok-style` |
| `crates/reactive` | `morok-reactive` | The signal graph: signals, memos, effects, scopes, one thread. | — |
| `crates/core` | `morok-core` | The element tree: style resolution, layout through Taffy, damage, the display list, input, focus, the AccessKit tree, text editing, animation, drag sessions, scrolling. Reaches the system through ports. | `morok-style` |
| `crates/platform` | `morok-platform` | objc2: windows and overlay windows, the Metal 4 renderer and its shaders, Core Text, atlases, images, canvases, text input, menus, the pasteboard, system drags, the AccessKit adapter. The only crate with `unsafe`. | `morok-core` |
| `crates/components` | `morok-components` | The component model (`view`: `Mount`, `ElementBuilder`, `Surfaces`) and the components with their `.css`. | `morok-core`, `morok-reactive`; builds with `morok-css` |
| `crates/morok` | `morok` | The composition root and the facade an application depends on: `Application`, `Window`, `Styles`, the layers as modules. | all of the above |

Unpublished: `crates/catalog` (`morok-catalog`, the catalog application, built on `morok` alone),
`crates/teststand` (`morok-teststand`, the Chrome-vs-MOROK stand), `crates/bench` (`morok-bench`,
the budget gates) and `crates/xtask` (formatting, clippy and comment rules).

## 3. Style schema

- **Tokens.** Seven families (`color-`, `space-`, `radius-`, `font-`, `text-`, `shadow-`,
  `motion-`) compile into typed IDs: a `Token<V>` is an index into the table of its kind of value
  in a `Theme`. A tokens file holds a light `:root` and a dark one; it compiles into one `Theme`
  per color scheme with the same IDs, so switching the scheme swaps a table and restyles —
  nothing is recompiled.
- **Style sheet.** Every class has the indices of its rules; a rule is a compound selector (classes,
  states, at most one child step) with its declarations, ordered by specificity then source
  order. Values that come from tokens keep the token ID and are resolved against the theme of
  the tree at style resolution. `@keyframes` compile into keyframes rules.
- **Generated code.** A `build.rs` calls `morok_css::build_script`, which writes `tokens.rs`
  (the `Tokens` struct of IDs, `TOKENS`, `LIGHT`, `DARK`) and `styles.rs` (the `Classes` of IDs,
  `CLASSES`, `STYLE_SHEET`) into `OUT_DIR`. The code reaches the schema through `::morok_style`
  inside the kit and `::morok::style` in applications; golden files pin the generated source.
- **Applications.** `build_script::generate_application` compiles an application over the kit:
  its tokens file defines every token of the kit, which keeps its ID, and may add tokens of its
  own, which get IDs after the kit's; the stylesheets of the kit come first, so their classes keep
  the IDs the components know them by. Every further tokens file is a set of values the
  application switches to: it defines exactly the tokens of the first and generates its `LIGHT`
  and `DARK` alone. `Application::set_themes` restyles every surface with them.

## 4. Surfaces and the application

- A **surface** owns one tree root, its layout and its drawable. The main window, every further
  window and every overlay (popover, tooltip, menu, the dropdown of select and combobox) are
  surfaces; `Surfaces` holds them by `SurfaceId`, with one signal graph serving them all. Toasts,
  dialogs and sheets are no surfaces of their own: they are drawn in the tree of the surface they
  belong to.
- The **application** owns the signal graph, the surfaces, the Core Text shaper and the shared GPU
  resources (device, pipelines, atlases, image textures). A window is a platform window whose
  content is a surface of the application; overlays are borderless windows placed against an
  anchor, flipped at screen edges, their trees owned by the component that opened them.
- **Frame stages**, per surface, in a fixed order: hover under a resting pointer, the signals
  that follow scrolls, the due timers, the flush of effects, style, the sizes of the pictures,
  layout, the display list, rendering, the AccessKit delta. The order up to the display list has
  one owner, two nested runs: `ElementTree::run_frame` runs the stages of a tree (style, the
  sizes of the pictures, layout, the display list), and `Surfaces::run_frame` runs the stages of
  a surface (hover, scrolls, timers, the flush) and then the run of its tree, unless the flush
  closed the surface. The stages are visible to their own crate alone: the host, the bench, the
  stand and the test stages call the runs, so none of them can leave a stage out or call one out
  of order. `ElementTree::lay_out` runs the stages of a tree up to layout, for an overlay laid
  out before its window opens. A caller that times the stages or counts what they allocate
  passes a `StageObserver`, which hears the end of every stage; the host passes `Unobserved`.
  The display link starts on damage or an active animation and stops after a frame that leaves
  neither.
- **Canvases.** A `Canvas` element hands its content box to the painter the application names;
  one painter draws the canvases of every window, through a `CanvasEncoder` that offers the fills
  of the kit (rectangles, ring sectors) and the Metal encoder, and carries the theme of the
  surface, so a canvas follows the theme and the token set as the boxes do.
- **Images.** An application adds a picture as its encoded bytes,
  `Application::add_image(Vec<u8>) -> ImageId`; the renderer decodes it off the main thread and
  `Application::image_status` tells whether it is still decoding, decoded at its natural size or
  unreadable. The `Image` component shows an `ImageId`, fitted by the `object-fit` of its class:
  a box of its CSS size while the picture decodes, the natural size once it is decoded, the box
  and its name when it is unreadable. Bytes are the one way in — the application reads files
  itself, and the same picture shown in every window is decoded once.
- **Closing.** The button of the frame closes a window: its surface view hears
  `NSWindowWillCloseNotification`, the content learns that the window closed, its overlays go,
  its frame loop stops and the menu bar forgets its menus. The handler of `Window::on_closed`
  runs on the next turn of the run loop, so that it may drop the `Window`; a closed window
  closes nothing twice when it is dropped. The menu bar shows the menus of the key window,
  synced on the first frame after the keyboard moves.
- **Handlers** run in the middle of an event; what opens or drops a window, or switches the
  appearance or the token set, is handed to `Application::defer` and runs once the event is over.

## 5. Element tree

- Nodes live in an arena and are addressed by ID handles (generational index): no
  `Rc<RefCell>` graphs; references to parent/children/focus are IDs.
- A node holds: a role tag (`Element`, `Text`, `Image`, `Icon`, `Canvas`, `Scroll`), a set of
  classes and state flags (`hover`, `focus-visible`, `active`, `disabled`, `checked`, …), the
  computed style, the layout result, damage flags, and an AccessKit node.
- Computed style = the class's static rule + state overlays + inline overrides from signals. A
  state change (mouse hover) flips a flag; it does not recompute a cascade.
- Taffy computes layout only for subtrees marked dirty; text measurement is cached by
  (string, style, width).

## 6. Upper layer: signals

- A component is a function invoked once at mount; it creates lower-layer nodes and subscribes
  their properties to signals.
- A signal change schedules exactly the effects that depend on it; effects write into nodes and
  mark damage. There is no tree diffing.
- Conditional and list rendering (`Show`, `For`) manage subtrees by key; large collections
  (`List`, `Table`, `Tree`) do not create a node per row — they keep a pool of visible rows and
  reuse it while scrolling.

## 7. Frame

```
event ─► signals/effects ─► style(flags) ─► layout(dirty) ─► display list(damage) ─► Metal
                                                           └─► AccessKit update (delta)
```

| Stage | Budget | How it is held |
|---|---|---|
| Events and effects | < 1 ms | targeted updates, no diff |
| Layout | < 1 ms | dirty subtrees only, measurement cache |
| Display list | < 1 ms | damage regions only, no allocations |
| Metal encoding | < 1 ms | instancing, buffers written into `MTLBuffer` directly |
| **CPU total** | **≤ 4 ms** | 120 fps on ProMotion with headroom for the app |

- While idle the display link is stopped: no damage — no frame.
- Animations and springs hold the display link only while active and release it when done.
- Input: keypress → glyph on screen within one frame (effects and redraw in the same tick).

## 8. Rendering

- Primitives are instanced quads: SDF rectangle (per-corner radius, border, shadows, gradients),
  glyph from the atlas, icon from the atlas, image, `Canvas` region.
- Clipping: rectangular — scissor; rounded (`overflow: hidden` + `border-radius`) — an SDF mask in
  the shader.
- Effects (group `opacity`, `filter`, `backdrop-filter: blur`, `mix-blend-mode`) — an offscreen
  pass for the subtree, only when the effect is set.
- `transform` — a matrix on the subtree's instances; layout does not depend on transforms (as in
  CSS).
- Translations in `f64`: a scroll offset, the displacement of a virtualized row and the
  translation of a `Transform` are double precision, while the linear part of a transform and
  every laid-out box stay `f32`. Past 2^24 px an `f32` steps by 2 px; the offset of a million
  rows lies there, and a row is placed by subtracting two such values, which `f64` does exactly.
  The renderer narrows the composed translation to `f32` for Metal, where it is small for
  anything on screen, so a one-pixel scroll moves the rows by one pixel at any depth.
- `Canvas` — the app receives an encoder and a rectangle; the kit guarantees z-order and clipping.

## 9. Text

- Shaping, line breaking, bidi, fallback fonts — Core Text (the same engine Safari uses on the Mac).
- Glyph atlas: lazily filled, keyed by (font, glyph, size, screen scale, subpixel phase).
- CSS metrics are reproduced exactly: `line-height`, baseline position, `letter-spacing`, OpenType
  features, `text-decoration`, `text-overflow: ellipsis`, `white-space`.
- Input: `NSTextInputClient` — marked text (IME), selection, dictation; undo/redo, clipboard.

## 10. Windows and the AppKit boundary

- Main window: `NSWindow` + an `NSView` with a `CAMetalLayer`. The frame and traffic lights are
  system-provided.
- Blur behind the window — `NSVisualEffectView` under a transparent Metal layer.
- Overlays from etalons (`Popover`, `Tooltip`, `Menu`, `Select`, `Combobox`) — a separate
  borderless window with its own Metal layer, positioned relative to its anchor, able to extend past
  the main window and the screen edge (flipping sides).
- `ToastStack`, `Dialog` and `Sheet` stay inside the window: toasts stack in the bottom right
  corner of its surface, a dialog and a sheet stand over a scrim that covers it.
- `ContextMenu`: `native` mode (`NSMenu`, the default) or `custom` (a Metal window).
- Menu bar, file dialogs, printing — system APIs.

## 11. Accessibility and keyboard

- Every node with a role exposes an AccessKit node; the tree is updated with deltas in the same
  frame as the pixels.
- Focus: traversal in tree order, a `tabindex`-like attribute, focus traps in `Dialog`.
- Focus ring — `:focus-visible` in the component's CSS (as in browsers).
- Keyboard shortcuts — registered at window and component level; the menu bar receives the same
  shortcuts.

## 12. Test stand

- **Suites and cases.** `crates/teststand` holds suites by name; a suite is the tokens it is styled
  with, the stylesheets of the components its cases mount, its own stylesheets and its cases. A
  case builds an element tree on a scene — boxes and texts, components of the kit, the screens of
  the catalog — at a viewport, at rest or at a moment of its motion, and may fill its canvases with
  shapes.
- **Compilation.** The tokens of a suite compile over the tokens of the kit, the stylesheets of the
  kit and then those of the suite compile against them, as an application compiles: both sides
  read the same tree.
- **Chrome side.** The pinned Chrome for Testing build (`crates/teststand/chrome.toml`) renders a
  harness page generated from the tree: the reset, the tokens as they are, the stylesheets with
  state pseudo-classes rewritten to marker classes, and one element per node — a `div` per box
  and text, an inline SVG per icon, an `img` per image, an SVG of paths per canvas. Scripts of the
  page place the carets of edited texts, scroll the `Scroll` elements and size the thumbs of the
  kit, and play the motion of a case. Screenshots are taken at scale 2 on a transparent background
  in both schemes, with the boxes of the elements.
- **Etalons.** `crates/teststand/etalons/<suite>/<case>/` holds the PNG of each scheme, the boxes
  and a manifest with the hashes of the stylesheets, the markup, the tokens and the Chrome build.
  `etalons render` writes them, `etalons verify` renders them again and fails on any difference,
  `check` compares MOROK with them and needs no Chrome; `all` runs a command over every suite.
- **MOROK side and the gates.** The same tree runs the stages of a frame and is drawn offscreen by
  the renderer of the kit. The geometry gate holds every box within 0.5 px of Chrome's; the color
  gate counts the pixels that are not anti-aliasing whose CIEDE2000 difference exceeds 2.0 and
  fails above 0.5 % of the area or a mean above 1.0 (`crates/teststand/thresholds.toml`). A
  failing case writes a diff PNG under `target/teststand/`.
- **Benchmarks.** `crates/bench` gates the budgets of §7 with failing exit codes
  (`crates/bench/budgets.toml`): `frame` on the reference scene of 2,000 boxes and on the two
  screens of the catalog, `idle`, `cold-start`, `input-latency`, `scroll-1m` (the table of a
  million rows) and `scroll-list` (the list of ten thousand documents) on the catalog itself;
  `all` runs every scenario.

## 13. Platform and publishing

- macOS 26 Tahoe+, Apple Silicon, Metal 4.
- License MIT/Apache-2.0, crates.io: `morok`.

## 14. Reference: Facet

MOROK does not fork Facet, but Facet's rendering layer is the proven prior art for §8–§9 and for
the snapshot-testing approach of §12. Read it as a reference, port ideas, never copy its core model.

- Repository: https://github.com/supostat/Facet
- Local checkout on the author's machine: `/Users/ingvar/Projects/home/Facet` — agents of the
  development pipeline read the reference from here; other contributors use the GitHub URL.
- Pinned commit: `f983588d0d238b6530ffba00e521fde511efb534` (2026-06-24) — read at this commit,
  later changes are not part of the reference.

| What to take | Facet files | MOROK target |
|---|---|---|
| SDF rounded rect with borders and soft shadows, instanced | `src/shaders/sdf.metal`, `src/platform/Instance.swift` | §8 rect primitive — the `.metal` shader ports almost as is; instance layout is redesigned for per-corner radius, gradients and clipping |
| Glyph quads from the atlas | `src/shaders/glyph.metal` | §8 glyph primitive |
| Lazy Core Text glyph atlas, shelf packing, rasterization | `src/platform/Atlas.swift`, `ShelfPacker.swift`, `GlyphRasterizer.swift`, `AtlasInspector.swift` | §9 glyph atlas — rewritten in Rust (objc2 + Core Text); add screen scale and subpixel phase to the key |
| Shaping and text measurement through Core Text | `src/platform/Shaper.swift`, `TextLayout.swift` | §9 shaping — called directly from Rust, no callback seam |
| Renderer and frame loop (`CADisplayLink` from the view, ProMotion-aware) | `src/platform/Renderer.swift`, `FrameLoop.swift`, `Window.swift` | §7 frame — add damage-driven redraw and an idle-stopped display link (Facet redraws every tick) |
| PNG reference snapshots and comparison | `tests/snapshots/SnapshotComparator.swift`, `RendererSnapshotTests.swift`, `TextSnapshotTests.swift` | §12 test stand — extended from self-references to Chrome-vs-MOROK perceptual diff |

**Do not take:** the Zig core (`src/core/*` — shadow tree, flexbox subset layout, dirty classifier,
binary mutation protocol), the JavaScriptCore bridge and `src/ui/mutationCodec.ts`. Their model (a
passive tree driven by React, a 32-bit prop enum frozen as a wire format) is exactly what MOROK
replaces; layout comes from Taffy instead.
