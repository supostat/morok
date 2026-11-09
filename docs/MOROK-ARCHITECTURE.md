# MOROK — Architecture

*Morok* (Russian: морок) is a glamour, an illusion — you see what is not there. The interface looks
like CSS from a browser, yet there is no browser — only Rust, Core Text and Metal. This document
describes how the kit is built; decisions and their rationale are in `docs/GRILL-morok.md`, the
visual language and the catalog in `design/morok-design-book.html`, prior art in §11 (Facet).

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

## 2. Element tree

- Nodes live in an arena and are addressed by ID handles (generational index): no
  `Rc<RefCell>` graphs; references to parent/children/focus are IDs.
- A node holds: a role tag (`Element`, `Text`, `Image`, `Icon`, `Canvas`, `Scroll`), a set of
  classes and state flags (`hover`, `focus-visible`, `active`, `disabled`, `checked`, …), the
  computed style, the layout result, damage flags, and an AccessKit node.
- Computed style = the class's static rule + state overlays + inline overrides from signals. A
  state change (mouse hover) flips a flag; it does not recompute a cascade.
- Taffy computes layout only for subtrees marked dirty; text measurement is cached by
  (string, style, width).

## 3. Upper layer: signals

- A component is a function invoked once at mount; it creates lower-layer nodes and subscribes
  their properties to signals.
- A signal change schedules exactly the effects that depend on it; effects write into nodes and
  mark damage. There is no tree diffing.
- Conditional and list rendering (`Show`, `For`) manage subtrees by key; large collections
  (`List`, `Table`, `Tree`) do not create a node per row — they keep a pool of visible rows and
  reuse it while scrolling.

## 4. Frame

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

## 5. Rendering

- Primitives are instanced quads: SDF rectangle (per-corner radius, border, shadows, gradients),
  glyph from the atlas, icon from the atlas, image, `Canvas` region.
- Clipping: rectangular — scissor; rounded (`overflow: hidden` + `border-radius`) — an SDF mask in
  the shader.
- Effects (group `opacity`, `filter`, `backdrop-filter: blur`, `mix-blend-mode`) — an offscreen
  pass for the subtree, only when the effect is set.
- `transform` — a matrix on the subtree's instances; layout does not depend on transforms (as in
  CSS).
- `Canvas` — the app receives an encoder and a rectangle; the kit guarantees z-order and clipping.

## 6. Text

- Shaping, line breaking, bidi, fallback fonts — Core Text (the same engine Safari uses on the Mac).
- Glyph atlas: lazily filled, keyed by (font, glyph, size, screen scale, subpixel phase).
- CSS metrics are reproduced exactly: `line-height`, baseline position, `letter-spacing`, OpenType
  features, `text-decoration`, `text-overflow: ellipsis`, `white-space`.
- Input: `NSTextInputClient` — marked text (IME), selection, dictation; undo/redo, clipboard.

## 7. Windows and the AppKit boundary

- Main window: `NSWindow` + an `NSView` with a `CAMetalLayer`. The frame and traffic lights are
  system-provided.
- Blur behind the window — `NSVisualEffectView` under a transparent Metal layer.
- Overlays from etalons (`Popover`, `Tooltip`, `Menu`, `Select`, `Combobox`, `Toast`) — a separate
  borderless window with its own Metal layer, positioned relative to its anchor, able to extend past
  the main window and the screen edge (flipping sides).
- `ContextMenu`: `native` mode (`NSMenu`, the default) or `custom` (a Metal window).
- Menu bar, file dialogs, printing — system APIs.

## 8. Accessibility and keyboard

- Every node with a role exposes an AccessKit node; the tree is updated with deltas in the same
  frame as the pixels.
- Focus: traversal in tree order, a `tabindex`-like attribute, focus traps in `Dialog`.
- Focus ring — `:focus-visible` in the component's CSS (as in browsers).
- Keyboard shortcuts — registered at window and component level; the menu bar receives the same
  shortcuts.

## 9. Test stand

- Every component has HTML markup equivalent to its tree, and the same `.css`.
- The stand renders both versions (headless Chrome and MOROK offscreen into a texture) for all
  states and themes, compares them by perceptual diff under a threshold, and stores a diff PNG.
- Benchmarks for the budgets in §4 are part of CI: frame, idle, scrolling 1M rows, cold start,
  input latency.

## 10. Platform and publishing

- macOS 26 Tahoe+, Apple Silicon, Metal 4.
- License MIT/Apache-2.0, crates.io: `morok`.

## 11. Reference: Facet

MOROK does not fork Facet, but Facet's rendering layer is the proven prior art for §5–§6 and for
the snapshot-testing approach of §9. Read it as a reference, port ideas, never copy its core model.

- Repository: https://github.com/supostat/Facet
- Local checkout on the author's machine: `/Users/ingvar/Projects/home/Facet` — agents of the
  development pipeline read the reference from here; other contributors use the GitHub URL.
- Pinned commit: `f983588d0d238b6530ffba00e521fde511efb534` (2026-06-24) — read at this commit,
  later changes are not part of the reference.

| What to take | Facet files | MOROK target |
|---|---|---|
| SDF rounded rect with borders and soft shadows, instanced | `src/shaders/sdf.metal`, `src/platform/Instance.swift` | §5 rect primitive — the `.metal` shader ports almost as is; instance layout is redesigned for per-corner radius, gradients and clipping |
| Glyph quads from the atlas | `src/shaders/glyph.metal` | §5 glyph primitive |
| Lazy Core Text glyph atlas, shelf packing, rasterization | `src/platform/Atlas.swift`, `ShelfPacker.swift`, `GlyphRasterizer.swift`, `AtlasInspector.swift` | §6 glyph atlas — rewritten in Rust (objc2 + Core Text); add screen scale and subpixel phase to the key |
| Shaping and text measurement through Core Text | `src/platform/Shaper.swift`, `TextLayout.swift` | §6 shaping — called directly from Rust, no callback seam |
| Renderer and frame loop (`CADisplayLink` from the view, ProMotion-aware) | `src/platform/Renderer.swift`, `FrameLoop.swift`, `Window.swift` | §4 frame — add damage-driven redraw and an idle-stopped display link (Facet redraws every tick) |
| PNG reference snapshots and comparison | `tests/snapshots/SnapshotComparator.swift`, `RendererSnapshotTests.swift`, `TextSnapshotTests.swift` | §9 test stand — extended from self-references to Chrome-vs-MOROK perceptual diff |

**Do not take:** the Zig core (`src/core/*` — shadow tree, flexbox subset layout, dirty classifier,
binary mutation protocol), the JavaScriptCore bridge and `src/ui/mutationCodec.ts`. Their model (a
passive tree driven by React, a 32-bit prop enum frozen as a wire format) is exactly what MOROK
replaces; layout comes from Taffy instead.
