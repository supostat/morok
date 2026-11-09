# GRILL: MOROK — a macOS UI kit on Rust + Metal

MOROK is a macOS UI kit that renders the entire interface through Metal. It is a separate new
project, not an evolution of Facet (`supostat/Facet`): the core is designed from scratch. From Facet
only rendering approaches and shaders carry over (SDF rectangles with shadows and borders, a lazy
Core Text glyph atlas, shaping). The primary requirement is visual compatibility with the HTML/CSS
etalons produced by `/mneme:design`: a screen laid out in an etalon with CSS and token variables
must look indistinguishable in a MOROK app (perceptual diff under a threshold, not pixel-exact). The
second requirement is speed: 120 fps on ProMotion, zero work while idle, virtualized collections of
a million rows. Consumers are the author's own applications: Quartz Commander and the Norton
Commander clone (both move from Zig to Rust and use the kit in full) and DOMOVOI (Rust). The kit is
published as open source (MIT/Apache-2.0) on crates.io under the name `morok`.

Architecture and rationale — `docs/MOROK-ARCHITECTURE.md`. The visual language of the base theme,
the component catalog and screen mockups — `design/morok-design-book.html`; base theme tokens —
`design/system/tokens.css`.

<!-- Section headings and branch marks keep the literal markers of the /mneme:grill protocol
     template («Заблокированные решения», «отдать plan'у (нужен веер)», «отложено пользователем»)
     so /mneme:plan's GRILL-READ matches them; the content is in English. -->

## Knowledge (заготовка)

- **A new project, not Facet.** Facet's core model (a passive "shadow" tree under React, a flat
  32-bit prop enum frozen as the wire format) got in the way of native components with their own
  state. The main consumers are Metal applications, not TSX, so the core is designed anew. From
  Facet the kit takes the `.metal` shaders (SDF rounded rect, shadows, borders, glyphs) and the
  atlas and shaping approaches as a reference, not as code.
- **Facet reference.** https://github.com/supostat/Facet, local checkout
  `/Users/ingvar/Projects/home/Facet`, at the pinned commit
  `f983588d0d238b6530ffba00e521fde511efb534` (2026-06-24). Take: `src/shaders/sdf.metal`,
  `src/shaders/glyph.metal`, the atlas/shaping/renderer files in `src/platform/` and the snapshot
  approach in `tests/snapshots/`. Do not take: `src/core/*`, the JavaScriptCore bridge,
  `src/ui/mutationCodec.ts`. The file-by-file map is in `docs/MOROK-ARCHITECTURE.md` §11.
- **Rust, not Zig.** Deciding arguments: Taffy provides full CSS layout (Block, Flexbox, Grid,
  float, `calc()`); AccessKit provides accessibility with a macOS adapter; objc2 provides typed
  AppKit/Metal/Core Text bindings; the language is stable for a library meant to live for years;
  the code is written by an agentic pipeline, and models write Rust more reliably. Rust's
  downsides (the borrow checker in a UI graph, build times) are handled by architecture: an arena
  plus ID handles for the tree.
- **Yoga does not fit.** Yoga is a CSS subset, mostly flexbox; as of November 2026 Grid in Yoga
  exists only as a series of open pull requests.
- **Platform layer in Rust via objc2, no Swift.** The choice barely affects speed (Metal calls from
  Swift and from objc2 both go through `objc_msgSend`; the instance buffer is written straight into
  `MTLBuffer`), but pure Rust removes the core→Swift callback per text node, removes ARC from hot
  loops, keeps a single toolchain, and lets AccessKit attach to `NSView` directly. The newest AppKit
  APIs (Liquid Glass), if objc2 does not have them yet, are declared by hand.
- **Two UI layers.** The lower layer is a retained element tree (a DOM analogue): stable IDs,
  computed styles, layout via Taffy, AccessKit nodes, focus, the input caret, redraw of changed
  regions only. The upper layer is declarative components that update the lower layer. Hot paths
  (a list of 100k+ rows) may drive lower-layer nodes directly.
- **Upper-layer reactivity is signals** (the SolidJS/Leptos model), not description diffing as in
  React: a component function runs once, each signal knows the nodes that depend on it, and a
  signal change updates exactly those nodes with no re-invocation and no diff. Chosen for speed.
- **Styles are a CSS subset compiled at build time.** Component styles live in `.css` files; at
  build time they become static style tables — no parsing and no cascade at runtime. CSS from an
  HTML etalon carries over almost verbatim. Anything unsupported (descendant selectors,
  `:nth-child`, `:has`, etc.) is a build error, never silently ignored.
- **`/mneme:design` etalons are plain CSS with token variables** (`design/system/tokens.css`:
  `:root { --<family>-<name>: … }`, families `color-`, `space-`, `radius-`, `font-`, `text-`, the
  dark scheme in `@media (prefers-color-scheme: dark)` with the same names).
- **Tokens are the single source.** The app's `tokens.css` is read at build time and generates
  typed Rust constants for the light and dark themes. Design and kit cannot drift apart.
- **The kit is independent of Melete.** Only the `tokens.css` format is shared with Melete;
  Melete's primitive vocabulary (`Stack`/`Row`/`Text`/`Box`/`Element`) is not mirrored — the kit and
  Melete evolve independently.
- **The test stand compares the same CSS.** Since component styles are real `.css`, the same file
  with equivalent HTML markup is rendered in headless Chrome and compared with the MOROK render by
  perceptual diff under a threshold. This serves both as regression tests and as measurable
  compatibility.
- **Browsers themselves render through the GPU** (Chrome via Skia, Safari via Core Animation; on
  the Mac both end up in Metal), so everything CSS draws is reproducible. The hardest parts are
  layout (solved by Taffy) and text (Safari on the Mac uses the same Core Text, so a match is
  achievable: line-height, baseline, letter-spacing, OpenType features, text-decoration, ellipsis,
  wrapping). Pixel-exact equality is not attainable because of anti-aliasing and rounding — the
  goal is "visually indistinguishable".
- **The AppKit boundary.** Metal draws only inside its own window and cannot see the pixels of other
  windows. Therefore always native: the menu bar, the window frame and traffic lights, file dialogs
  and printing, blur of what is behind the window (`NSVisualEffectView` under a transparent Metal
  layer). Everything present in etalons (dropdowns, popovers, tooltips, custom menus) is drawn by
  Metal — each in its own borderless window so it can extend past the main window. The context menu
  is a component setting: native `NSMenu` by default, custom when needed.
- **Text editing is plain text only**, single-line and multi-line fields: caret, selection, IME via
  `NSTextInputClient` (marked text), system dictation, undo/redo, clipboard, line wrapping,
  scrolling inside the field. Rich text and a code editor are outside the kit.
- **Icons are SVG** — the same files as in the etalons; rasterized into an atlas per size and screen
  scale (e.g. via resvg). SF Symbols are not used: browsers do not know them, and the etalon and the
  app would diverge.
- **Animation** — `transition` and `@keyframes` in the same `.css` files with the same easing
  curves; plus spring physics for scrolling and gestures (momentum, rubber band, dragging).
- **Performance budgets** (verified by benchmarks in CI):
  - frame — 120 fps on ProMotion, the CPU part of a frame (update → layout → encode) ≤ 4 ms;
  - idle — zero frames and ~0% CPU when nothing changes;
  - scrolling a `Table` of 1,000,000 rows without dropped frames;
  - cold start — first window with content in < 150 ms;
  - input — keypress to glyph on screen within ≤ 1 frame.
- **Platform — macOS 26 Tahoe+, Apple Silicon only.** Metal 4 and Liquid Glass with no fallback
  paths; all consumers are the author's own applications.

## Conventions (заготовка)

- Kit code is Rust only; there is no Swift in the repository. Shaders are `.metal`.
- No C ABI: Zig consumers move to Rust.
- The lower layer is a retained tree in an arena with ID handles; no `Rc<RefCell<…>>` node graphs.
- The upper layer is signals; re-invoking a component function on data change is ruled out by the
  model.
- Colors, spacing, radii, typography, shadows and motion in component `.css` go through tokens only
  (`var(--…)`); a raw color is a build error. A component's own geometry (control height, border
  width) may be given in px.
- Unsupported CSS is a build error naming the file, line and construct.
- No heap allocations on the hot path of a frame; redraw by damage only; the display link does not
  tick while idle.
- Every component has: all states (`:hover`, `:focus-visible`, `:active`, `:disabled`, plus its own),
  an AccessKit node with a role and a name, keyboard navigation, and a snapshot test against Chrome.
- Theme: follows the system (light/dark) by default; the app may override it.

## Заблокированные решения

Locked decisions — closed in discussion on 2026-11-09, not to be silently revisited.

1. A separate new project; Facet is not evolved into the kit.
2. Core and platform layer language — Rust; platform via objc2, no Swift.
3. Two UI layers: a retained element tree + declarative components on top.
4. Upper-layer reactivity — signals.
5. Compatibility with `/mneme:design` HTML/CSS etalons is mandatory.
6. Component styles — a CSS subset compiled at build time.
7. Tokens — `tokens.css`, code generation of Rust constants (light + dark).
8. The kit is independent of Melete; only the token format is shared.
9. Layout — Taffy (Block, Flexbox, Grid, absolute, percentages, `calc()`).
10. Accessibility — AccessKit.
11. AppKit boundary: system parts native (menu bar, window frame, file dialogs, blur behind the
    window); everything from etalons — Metal in separate windows; the context menu is configurable
    per component.
12. Text — plain text only (single-line and multi-line fields).
13. Icons — SVG in an atlas; SF Symbols are not used.
14. Animation — CSS `transition` + `@keyframes` + springs for scrolling and gestures.
15. Zig applications (Quartz Commander, the NC clone) move to Rust; no C ABI.
16. Performance budgets — as in Knowledge.
17. macOS 26 Tahoe+, Apple Silicon only.
18. Open source MIT/Apache-2.0, crates.io, name `morok`.
19. Kit contents:
    - base elements: `Element`, `Text`, `Image` (PNG/JPEG/HEIC, asynchronous decoding), `Icon`,
      `Scroll`, `Canvas` (custom Metal drawing inside the tree);
    - controls: `Button`, `IconButton`, `Checkbox`, `Radio`, `Switch`, `Slider`,
      `SegmentedControl`, `TextField`, `TextArea`, `SearchField`, `Select`, `Combobox`,
      `ProgressBar`, `Spinner`;
    - collections (all virtualized): `List`, `Table` (column resizing, sorting, reordering),
      `Tree`, `Tabs`;
    - window scaffolding: `SplitView`, `Sidebar`, `Toolbar`, `StatusBar`;
    - overlays: `Popover`, `Tooltip`, `Menu`, `ContextMenu`, `Dialog`/`Sheet`, `Toast`;
    - cross-cutting mechanisms: focus and keyboard navigation, keyboard shortcuts, drag-and-drop
      (including files Finder ↔ app), clipboard, light/dark theme, multiple windows.

## Открытые ветки

Open branches.

- Workspace crate layout (core / platform / styles / components / test stand) → отдать plan'у
  (нужен веер).
- Signals implementation: custom or an existing reactive graph (e.g. `reactive_graph` from Leptos)
  → отдать plan'у (нужен веер).
- CSS compiler: `build.rs` vs a procedural macro; the parser (`cssparser` / `lightningcss`); the
  exact grammar of the supported subset → отдать plan'у (нужен веер).
- Taffy integration: the MOROK tree implements Taffy's traits directly vs a `TaffyTree` alongside →
  отдать plan'у (нужен веер).
- Atlas strategy (glyphs, icons, images): shared or separate, eviction → отдать plan'у (нужен веер).
- Test stand runner: how to drive headless Chrome and which perceptual diff metric → отдать plan'у
  (нужен веер).
- Token families `shadow-` and `motion-`: the base theme (`design/system/tokens.css`) introduces
  them beyond the five `/mneme:design` families (`color-`, `space-`, `radius-`, `font-`, `text-`) —
  legitimize them in the `design` skill or express shadows and motion differently → отдать plan'у
  (нужен веер).
- Phase order and `until` boundaries for implementing the kit → отдать plan'у (нужен веер).
- Porting Quartz Commander and the NC clone to MOROK → отложено пользователем (separate tasks after
  the kit).
