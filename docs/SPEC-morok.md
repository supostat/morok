<!--
  SPEC-morok.md — the first spec of the project: the `morok` UI kit for macOS on Rust + Metal, from
  an empty repository to the component catalog, the Chrome-vs-MOROK test stand and the budget
  benchmarks. 24 phases.

  DECISION BOUNDARY (fan of 2026-11-09, variant A chosen on the recommendation): the kit owns its
  hot path — synchronous single-threaded signals, Taffy's traits on the kit's own arena, a whitelist
  CSS compiler on `cssparser` driven from `build.rs`, atlases split by pixel format — in a layered
  workspace, with the test stand and the benchmarks in place before the catalog.
  Rejected: B — reuse first (`reactive_graph`, a `TaffyTree` alongside, `lightningcss` behind a proc
  macro, one RGBA atlas, a Playwright script, layer-by-layer order): the `reactive_graph` README
  schedules effects as asynchronous tasks on a process-wide executor, which works against "keypress
  to glyph within one frame"; two trees need synchronising; parity and budgets are first measured
  after every layer is built. C — A's engine in two published crates with a Playwright stand:
  layering held by discipline instead of the compiler, a second toolchain, Chrome on every run.
-->

# Baseline

- Prior spec: none — the first spec of the project.
- Prior spec-hash: sha256:<placeholder — confirm>
- Sources of truth: `docs/GRILL-morok.md` (the protocol — its 19 locked decisions and its Knowledge
  and Conventions seeds are given and not reopened here), `docs/MOROK-ARCHITECTURE.md`,
  `design/morok-design-book.html` (the visual etalon of the base theme and the catalog),
  `design/system/tokens.css` (the canonical tokens).
- Exists and is not touched: those four files and `CLAUDE.md`. The repository holds no code: no
  `Cargo.toml`, no CI, no scripts.
- The protocol's eight open branches are closed here: crate layout (Stack), signals, the CSS
  compiler and its grammar, Taffy integration, atlases, the test stand, the token families
  (Knowledge), phase order (Gameplan).
- Out of scope: porting Quartz Commander and the Norton Commander clone (deferred by the user in the
  protocol); publishing to crates.io; changing the `design` skill of the mneme plugin; registering a
  CI runner — budgets are gated by commands with failing exit codes.
- Version: no release automation was found (no `.github/`, no release script, no manifest), so
  there is no version-bump phase; the workspace is created at `0.1.0`.
- References: Facet at the pinned commit `f983588d0d238b6530ffba00e521fde511efb534` (file map in
  `docs/MOROK-ARCHITECTURE.md` §11) for shaders, shelf packing and shaping; DOMOVOI
  (`/Users/ingvar/Projects/home/DOMOVOI`) for the Rust side — workspace conventions,
  `crates/render/build.rs`, `harness = false` tests. Both are read, never copied wholesale.
- Rationale of the choice — staged decision note (id after accept).

# Stack

- Published crates, directory → package, each depending only on the ones named:
  - `crates/style` → `morok-style`: the schema of tokens and compiled styles. No dependencies.
  - `crates/css` → `morok-css`: the build-time compiler. Depends on `morok-style`.
  - `crates/reactive` → `morok-reactive`: the signal graph. No workspace dependencies.
  - `crates/core` → `morok-core`: element tree, style resolution, layout, damage, display list,
    input, focus, accessibility tree, text model, animation. Depends on `morok-style`.
  - `crates/platform` → `morok-platform`: objc2 — windows, Metal 4 renderer, Core Text, atlases,
    text input, overlay windows, the AccessKit adapter. Depends on `morok-core`.
  - `crates/components` → `morok-components`: the component model (`view`) and the kit's
    components with their `.css`. Depends on `morok-core`, `morok-reactive`; builds with `morok-css`.
  - `crates/morok` → `morok`: the composition root and the public facade. Depends on all of them.
- Unpublished crates: `crates/teststand` → `morok-teststand`, `crates/bench` → `morok-bench`,
  `crates/catalog` → `morok-catalog`, `crates/xtask` → `xtask`.
- Third-party crates: the objc2 family (`objc2`, `objc2-foundation`, `objc2-app-kit`, `objc2-metal`,
  `objc2-quartz-core`, `objc2-core-text`, `objc2-core-graphics`, `objc2-core-foundation`, `block2`,
  `dispatch2`); `taffy`; `accesskit` and `accesskit_macos`; `cssparser`; `quote`, `proc-macro2`,
  `prettyplease` for code generation; `resvg`; `bytemuck`. Stand only: `chromiumoxide`, `tokio`,
  `png`.
- Tools: the Metal toolchain of Xcode through `/usr/bin/xcrun`; a pinned Chrome for Testing build.

# Conventions

- Given by the protocol and unchanged: Rust only, shaders in `.metal`, no C ABI; the tree is an
  arena with ID handles; the upper layer is signals; component `.css` takes colors, spacing, radii,
  typography, shadows and motion from tokens only, its own geometry may be in px; unsupported CSS is
  a build error naming file, line and construct; no heap allocation on the hot path of a frame;
  redraw by damage only; the display link does not tick while idle; every component has all its
  states, an AccessKit node with a role and a name, keyboard navigation and a snapshot test against
  Chrome; the theme follows the system and the application may override it.
- Project rules of `CLAUDE.md`: comments are English, rare, and never mention phases, tasks or the
  spec; everything under `docs/` is English.
- Workspace as in DOMOVOI: `crates/<name>`, edition 2024, the toolchain pinned in
  `rust-toolchain.toml`, every dependency pinned exactly in `[workspace.dependencies]`,
  `[workspace.lints]` with `unsafe_op_in_unsafe_fn` denied, clippy `pedantic` as warnings and
  `dbg_macro`, `todo`, `unimplemented` denied; `rustfmt.toml` with `max_width = 120`.
- A crate is created by the phase that fills it; there are no placeholder crates.
- `unsafe` lives only in `morok-platform`; every other published crate carries
  `#![forbid(unsafe_code)]`.
- A CSS construct enters the compiler in the phase that renders it and adds its stand case. Until
  then it is a build error.
- A component is two files side by side: `crates/components/src/<name>.rs` and
  `crates/components/src/<name>.css`.
- A test that touches AppKit is a `harness = false` target that runs on the main thread.
- Stand etalons are produced only by the stand's `etalons render` command from the pinned Chrome and
  are never edited by hand.
- The stand thresholds and the benchmark budgets are fixed by this spec. A phase does not change
  them; a case that cannot meet them is a defect of the kit.

# Knowledge

- **Layering.** `morok-core` knows nothing of AppKit and reaches the system through ports that
  `morok-platform` implements: text measurement and shaping, the surface host (main and overlay
  windows), the clipboard, the drag session, native menus, frame scheduling. `morok-components`
  depends on the ports, never on the platform. `morok` wires the adapters in.
- **Style schema is the contract.** The types of `morok-style` are both what the compiler builds in
  memory and what the generated tables are made of, const-constructible. A test asserts that the
  generated source equals a committed golden file, that the golden file compiles, and that it equals
  the in-memory result. The generated code addresses the schema through a configurable path:
  `::morok_style` inside the workspace, `::morok::style` in applications.
- **Token contract.** Seven families, each admitting fixed value kinds: `color-` a color; `space-`
  and `radius-` a px length; `font-` a family list; `text-` a px size, or a unitless leading when the
  name starts with `text-leading`, or an integer weight when it starts with `text-weight-`;
  `shadow-` a comma-separated list of layers with an optional `inset`; `motion-` a duration (`ms`,
  `s`) or an easing (`cubic-bezier()`, `steps()`, `linear`), told apart by value syntax. Raw colors
  are legal only in the tokens file. A tokens file holds one `:root` block and one
  `@media (prefers-color-scheme: dark) { :root { … } }` block with the same names. Tokens are
  referenced by typed IDs and resolved against the active theme, so switching the theme swaps a
  table and restyles; nothing is recompiled. The Rust name is the full token name in snake case
  (`--space-1` → `space_1`). `shadow-` and `motion-` are the kit's own families; the mneme lint
  accepts them because its `parseTokens` reads every custom property.
- **Grammar.** The target is the table in section VI of the design book, made exact here.
  Selectors: a class, a compound class (`.button.primary`), state pseudo-classes on either, one
  child step between two such selectors (`.table > .tr`), comma lists, and the pseudo-elements
  `::placeholder` and `::selection`. States: `:hover`, `:active`, `:focus`, `:focus-visible`,
  `:focus-within`, `:disabled`, `:checked`. At-rules: `@keyframes` only.
  Layout: `display` block, flex, grid, none; `position` relative, absolute; the flex and grid
  properties; `gap`; `width`, `height`, min and max; `margin`, `padding`, `inset`; percentages and
  `calc()` over px and percentages; `overflow`.
  Box painting: `background-color`, `linear-gradient()` and `radial-gradient()` with token stops,
  `border` and per-corner `border-radius`, `box-shadow` with `inset`, `outline` and
  `outline-offset`, `color`.
  Text: `font-family`, `font-size`, `font-weight`, `font-style`, `line-height`, `letter-spacing`,
  `text-align`, `text-decoration`, `text-overflow`, `white-space`, `font-variant-numeric`.
  Effects: `opacity`; `transform` with `translate`, `translateX`, `translateY`, `scale`, `scaleX`,
  `scaleY`, `rotate`, and `transform-origin`; `filter` with `blur`, `brightness`, `contrast`,
  `grayscale`, `saturate`, `opacity`; `backdrop-filter` with `blur`, `brightness`, `saturate`;
  `mix-blend-mode` with `normal`, `multiply`, `screen`, `overlay`, `darken`, `lighten`.
  Motion: `transition`, `animation`, `@keyframes`, `cubic-bezier()`, `steps()`, `linear`.
  Values: `var(--token)`, px, percentages, unitless numbers, keywords, `transparent`,
  `currentColor`. Everything else is a build error: descendant selectors, a second child step,
  `:nth-child`, `:has`, `*`, attribute selectors, other pseudo-elements, raw colors, `!important`,
  `float`, `position: sticky` outside a `Scroll`, `url()`, `@font-face`, `@import`, `@media`, the
  units `em`, `rem`, `vw`, `vh`, `conic-gradient()`, 3D transforms, `drop-shadow()`.
- **Cascade and inheritance.** The same file runs through Chrome's cascade, so overlays are applied
  in cascade order: specificity first, source order second. A pseudo-class weighs as much as a
  class. `color`, the `font-*` properties, `line-height`, `letter-spacing`, `text-align` and
  `white-space` inherit from parent to child at style resolution; every other property starts from
  its CSS initial value under the stand's reset (`box-sizing: border-box`, zero margin and padding).
- **Diagnostics.** The format of the design book: `error[morok-css]: <message>`, then
  `--> <file>:<line>:<column>`, the source line, a caret under the construct and a hint. `cssparser`
  reports the location of every token.
- **Taffy.** The arena implements `TraversePartialTree`, `LayoutPartialTree` and `CacheTree`, and
  layout starts from `compute_root_layout`; `compute_child_layout` dispatches by node role to
  `compute_flexbox_layout`, `compute_grid_layout`, `compute_block_layout` or, for text and images,
  `compute_leaf_layout` with the measure port. `LayoutPartialTree` has an associated style type, so
  a wrapper around the resolved style implements Taffy's style traits and no `taffy::Style` is
  stored (the wrapper lives in `morok-core` because of the orphan rule). `calc()` goes through
  `resolve_calc_value`. Marking a node dirty clears the layout cache of the node and its ancestors —
  the kit's job once `TaffyTree` is not used. These traits change between Taffy's minor versions:
  the version is pinned.
- **Signals.** One thread, the main one; the graph is an arena with ID handles. Dependencies are
  tracked at run time and re-collected on every run of a memo or an effect. Propagation is
  glitch-free: in a diamond every effect runs once and sees consistent values. Writes mark and
  queue; `flush` runs the queue at a fixed stage of the frame. A scope owns the signals and effects
  created inside it and disposes them with its subtree.
- **Frame and surfaces.** Stage order: events, `flush` of effects, style, layout, display list,
  render, the AccessKit delta. A surface owns one tree root, its layout and its drawable; the main
  window and every overlay are surfaces, and several windows are several surfaces. The application
  owns the reactive runtime and the shared GPU resources (device, pipelines, atlases). The display
  link starts on damage or an active animation and stops after a frame that leaves neither.
- **Rendering.** Metal 4 API (`MTL4*`); `objc2-metal` 0.3.2 already carries its bindings. Facet and
  DOMOVOI are written against the earlier Metal API: port the ideas, not the calls. Browsers
  composite and interpolate gradients in non-linear sRGB with premultiplied alpha, so the frame
  buffer is not an `_srgb` format and colors are blended as sRGB values; a linear pipeline would
  fail the stand on every translucent edge. Rectangular clips use the scissor, rounded clips an SDF
  mask in the shader. A subtree gets an offscreen pass only when an effect that needs one is set.
- **Text.** Core Text shapes, breaks lines and picks fallback fonts. Rendering is grayscale
  anti-aliasing with subpixel positioning; the atlas key is font, glyph, size, screen scale and
  subpixel phase. The CSS line box comes from `line-height` with half-leading above and below, the
  baseline from the font's ascent. `-apple-system` and `system-ui` resolve to the system UI font.
  From memory, unverified — check on the first text case of the stand: Chrome on macOS thickens
  stems unless the page sets `-webkit-font-smoothing: antialiased`, so the stand's reset sets it.
- **Atlases.** Two atlases split by pixel format. The mask atlas holds glyphs and single-color
  icons, tinted in the shader, so an icon needs no entry per color. The color atlas holds color
  glyphs, multi-color icons and small images. Each is a set of shelf-packed pages; when the page
  budget is reached the least recently used page is dropped whole and its entries are rasterized
  again on demand. Large images are separate textures under a byte budget with LRU eviction.
  Facet's atlas only grows and re-rasterizes: it is the reference for shelf packing, not eviction.
- **Shader build.** `crates/platform/build.rs` runs `/usr/bin/xcrun -sdk macosx metal -c` and
  `metallib` into `OUT_DIR` and passes the library path through `cargo::rustc-env`, as
  `crates/render/build.rs` of DOMOVOI does. It needs Xcode's Metal toolchain installed.
- **AccessKit.** `accesskit_macos::Adapter::new` takes the view as a raw pointer, the initial focus
  state and an action handler, on the main thread. The kit declares its own view class, so it
  forwards `accessibilityChildren`, `accessibilityFocusedUIElement` and `accessibilityHitTest:` to
  the adapter itself; `SubclassingAdapter` is not needed. Updates go through `update_if_active`,
  and the returned events are raised on the main thread. The adapter brings its own objc2 version;
  its API is raw pointers, so the kit's pin does not have to match.
- **Test stand.** For one case the stand builds the component's element tree and renders it twice.
  The Chrome side is generated from that tree: one element per node, a harness page with the reset,
  `tokens.css` and the component's stylesheet. State pseudo-classes in the stylesheet are rewritten
  to marker classes and the markup carries the markers of the case — a class weighs as much as a
  pseudo-class, so the cascade is unchanged, and `:disabled` and `:checked` work on any element.
  Declarations stay byte-identical. Chrome is the build pinned in `crates/teststand/chrome.toml`,
  fetched into the user cache and launched through `chromiumoxide`
  (`BrowserConfig::builder().chrome_executable(…)`); the system Chrome is never used. A case is
  rendered at scale 2, on a transparent background, in the light and the dark scheme. The etalon of
  a case is a PNG, the boxes of its elements and a manifest with the hashes of the stylesheet, the
  markup, the tokens and the Chrome build. `etalons render` writes etalons, `etalons verify`
  re-renders them and fails on any difference, `check` compares MOROK with the committed etalons and
  needs no Chrome.
- **Stand thresholds.** Geometry gate: every element box within 0.5 px of Chrome's. Color gate, on
  pixels that are not anti-aliasing: a pixel mismatches when its CIEDE2000 difference exceeds 2.0; a
  case fails when mismatching pixels exceed 0.5 % of its area (the fraction Facet's comparator uses)
  or when the mean difference over the case exceeds 1.0. They live in
  `crates/teststand/thresholds.toml`.
- **Benchmarks.** Budgets live in `crates/bench/budgets.toml`: events, layout, display list and
  Metal encoding under 1 ms each and 4 ms in total; zero allocations per steady frame; zero frames
  over 10 s of idle; under 150 ms from process start to the first presented frame with content; the
  glyph of a keypress in the frame of its event; every frame of a scripted scroll through 1,000,000
  rows within the frame budget. A gate compares the 95th percentile of the measured frames after a
  warm-up. The reference scene of `frame` is 2,000 styled boxes and 500 text runs, with 1 % of the
  nodes changed per frame. Scenarios arrive with their phases: `frame` and `idle` with the harness,
  `cold-start` with the first component, `input-latency` with text input, `scroll-1m` with `Table`.
- **Applications and themes.** An application's `build.rs` compiles its own `tokens.css` and
  stylesheets through `morok-css`. Its tokens file must define every token of the base theme — a
  missing one is a build error — and its extra tokens get IDs after the kit's.
- **Comment rules — rule → the case that fails without it.**
  - a comment with Cyrillic letters fails → a `.rs` fixture with a Russian line comment
  - a comment naming a phase, task or step by number fails → a fixture with `// phase 3: retry`
  - a comment naming the spec or the gameplan fails → a fixture with `// see SPEC-morok`
  - `.rs`, `.metal` and `.css` under `crates/` are scanned → the same comment in a `.metal` and a
    `.css` fixture
  - a string literal is not a comment → a Cyrillic string literal that must pass
  - findings name file and line and the exit code is 1; a clean tree exits 0 → both runs
- **Token compiler — rule → the case that fails without it.**
  - an unknown family is an error → `--glow-1: 4px`
  - a value kind the family does not admit is an error → `--space-1: #fff`
  - a name present in one scheme only is an error → a token missing from the dark block
  - a name declared twice in one scheme is an error → two `--color-bg` in `:root`
  - `text-leading*` is a leading, `text-weight-*` a weight, any other `text-` a size → one of each
  - a `motion-` value is a duration or an easing by its syntax → `120ms` and `cubic-bezier(…)`
  - a shadow keeps its layers and `inset` → a two-layer shadow with an inset layer
  - a block other than the two `:root` blocks is an error → a `.card { … }` rule in the tokens file
  - a diagnostic carries file, line and column → an error on a known line and column
  - output is deterministic → two runs over the same file compared byte for byte
- **Stylesheet compiler — rule → the case that fails without it.**
  - a descendant selector is an error → `.list .row`
  - a second child step is an error → `.a > .b > .c`
  - `:nth-child`, `:has`, `*` and attribute selectors are errors → one fixture each
  - an unknown pseudo-class or pseudo-element is an error → `.a:visited`, `.a::before`
  - a raw color is an error → `color: #fff`
  - an unknown token is an error → `var(--color-nope)`
  - a token of the wrong kind for the property is an error → `gap: var(--color-bg)`
  - an unknown or not-yet-rendered property is an error → `float: left`
  - `!important` is an error → `color: var(--color-text) !important`
  - an at-rule other than `@keyframes` is an error → `@media`, `@import`, `@font-face`
  - a unit outside px and percent is an error → `padding: 1em`
  - overlays keep cascade order → two rules of equal specificity where the later must win, and a
    more specific earlier rule that must win over a later one
  - a shorthand equals its longhands → `padding: 0 var(--space-3)` against four longhands
- **Comparator — rule → the case that fails without it.**
  - a box more than 0.5 px away fails and names the node → a tree with one node shifted by 1 px
  - a missing or extra element fails → an etalon with one more box than the tree
  - anti-aliasing pixels do not count → two renders differing only along a rounded edge must pass
  - a pixel above ΔE 2.0 mismatches → a solid area shifted by ΔE 3
  - mismatches above 0.5 % fail → 1 % of the area shifted
  - a mean above 1.0 fails → the whole area shifted by ΔE 1.5
  - a manifest hash that differs fails as stale before comparing → a changed stylesheet
  - renders of different size fail → an etalon one pixel wider
  - a failing case writes a diff PNG → the file exists after a failure
  - any failing case gives a non-zero exit, none gives zero → both runs
  - both schemes are compared → a case that matches in light and differs in dark
- **Budget gate — rule → the case that fails without it.**
  - a 95th percentile above the budget exits 1 and names scenario and stage → a slowed stage
  - a value at or under the budget exits 0 → a fast scene
  - an allocation in a steady frame fails → a scene that allocates each frame
  - a frame during idle fails → a scene with a repeating timer
  - warm-up frames are not measured → a slow first frame that must not fail the gate
  - an unknown scenario name exits 2 → `bench nope`
- **Limits of the gates.** The gates prove the box tree, the pixels of an offscreen frame, the
  AccessKit tree and the CPU and allocation cost of scripted frames. They do not prove: smoothness on
  a ProMotion panel as the eye sees it, a real input method and dictation, a drag between Finder and
  the application, VoiceOver as a user experiences it. These are human acceptance at the `wire`
  boundary, against the design book: every component of sections IX–XIII in every state shown, both
  themes, one accent point per screen, sizes as in the book's CSS.
- **Publishing limit.** The canonical tokens file sits outside every crate directory; packaging for
  crates.io needs it inside `crates/components`. Publishing is out of scope and this is left open.

# Gameplan

<!-- 24 phases, a chain: each phase depends on the one before it. The mark in a title says what the
     phase takes from earlier ones: "consumes phase N" is real code consumption and comes with a
     criterion on the real module; "order-only after phase N" is ordering alone. The terminal
     `wire` phase assembles the main path from real modules. `cargo xtask check` is the project's
     typecheck gate and closes every code phase. -->

### Phase 1: workspace-tokens — workspace, tooling and the token compiler

- [ ] Workspace root: `Cargo.toml` (resolver 3, `[workspace.package]` at `0.1.0`, edition 2024,
  license `MIT OR Apache-2.0`, pinned `[workspace.dependencies]`, `[workspace.lints]`),
  `rust-toolchain.toml`, `rustfmt.toml`, `.cargo/config.toml` with the `xtask` alias,
  `LICENSE-MIT`, `LICENSE-APACHE`, `target` in `.gitignore`.
- [ ] `crates/style`, token half: colors, lengths, shadows, durations, easings, font families, typed
  token IDs and the theme tables — plain const-constructible data.
- [ ] `crates/css`, token compiler: parse a tokens file by the token contract, report diagnostics,
  generate the light and dark theme tables and the token IDs, and expose the entry a `build.rs`
  calls (writes into `OUT_DIR`, prints `cargo::rerun-if-changed`).
- [ ] `crates/xtask`: `cargo xtask check` — rustfmt check, clippy with warnings denied, and the
  comment rules over `crates/`.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok-css --test token_compiler
```
- `design/system/tokens.css` compiles into both themes and every token-compiler case of Knowledge behaves as listed (exit 0).
```timeout=900s
cargo test -p morok-css --test generated_tokens
```
- The generated source equals the golden file, the golden file compiles and equals the in-memory result (exit 0).
```timeout=900s
cargo test -p xtask --test comment_rules
```
- Every comment-rule case of Knowledge behaves as listed (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and comment rules are clean across the workspace (exit 0).

### Phase 2: css-compiler — style schema and the stylesheet compiler (consumes phase 1: token tables)

- [ ] `morok-style`, stylesheet half: class rules, compound and state overlays in cascade order,
  child rules, typed declarations for layout and box painting, token references, `calc()`.
- [ ] `morok-css`, stylesheet compiler: the selectors, states, values and shorthands of the grammar
  for layout, box painting and `color`; everything else is a diagnostic.
- [ ] Generation of static tables with one typed constant per class and a configurable schema path;
  the `build.rs` entry compiles a tokens file together with a directory of `.css`.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok-css --test stylesheet_compiler
```
- Selectors, states, child rules, shorthands and `calc()` compile into the expected schema values (exit 0).
```timeout=900s
cargo test -p morok-css --test subset_errors
```
- Every stylesheet-compiler case of Knowledge is a diagnostic with file, line and column (exit 0).
```timeout=900s
cargo test -p morok-css --test generated_tables
```
- The generated tables equal the golden file, the golden file compiles and equals the in-memory result (exit 0).
```timeout=900s
cargo test -p morok-css --test real_tokens_resolution
```
- A stylesheet using tokens of every family resolves against the real `design/system/tokens.css`, no stub tokens (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and comment rules are clean (exit 0).

### Phase 3: core-tree — element tree, style resolution, layout, damage and display list (consumes phase 2: compiled style tables)

- [ ] `crates/core`: the element tree — an arena with generational IDs, roles (`Element`, `Text`,
  `Image`, `Icon`, `Canvas`, `Scroll`), classes, state flags, child order.
- [ ] Style resolution: the class rule, overlays in cascade order, child rules, inline overrides,
  inheritance and the active theme; a state flip restyles the node and only the children its child
  rules name.
- [ ] Layout: Taffy's traits on the arena, text leaves measured through the port, layout recomputed
  for dirty subtrees only.
- [ ] Damage and display list: dirty flags become damage rectangles; box primitives (background,
  gradient, border, per-corner radius, shadows, outline, clip) are rebuilt only inside the damage,
  with no allocation once the buffers reach their steady size.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok-core --test style_resolution
```
- Cascade order, child rules, inheritance and theme switch hold on tables produced by the real `morok-css`, no hand-built tables (exit 0).
```timeout=900s
cargo test -p morok-core --test layout
```
- Flex, grid, block, absolute and `calc()` layouts match the expected boxes and only dirty subtrees are recomputed (exit 0).
```timeout=900s
cargo test -p morok-core --test damage_display_list
```
- A state flip damages only its rectangle and the display list changes only inside it (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and comment rules are clean (exit 0).

### Phase 4: metal-window — window, Metal renderer and frame loop (consumes phase 3: display list)

- [ ] `crates/platform`: application, window and a view with a `CAMetalLayer` through objc2; resize
  and backing-scale changes; the system appearance observed and overridable.
- [ ] Shaders built by `build.rs`; the Metal 4 device, queue and the rectangle pipeline — instanced
  SDF quads with per-corner radius, border, gradients, outer and inset shadows and a rounded clip
  mask; rectangular clips by scissor.
- [ ] The display list encoded into instance buffers written straight into `MTLBuffer`; an offscreen
  target that returns the frame as an RGBA image.
- [ ] Frame loop: a display link created by the view, started by damage and stopped after a frame
  that leaves none.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok-platform --test offscreen_boxes
```
- A display list built by the real `morok-core` renders offscreen with the expected fills, corners, borders and shadow falloff (exit 0).
```timeout=900s
cargo test -p morok-platform --test display_link_idle
```
- A window with content and no damage receives no display-link callback (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and comment rules are clean (exit 0).

### Phase 5: text — shaping, atlases and CSS text metrics (consumes phases 3-4: measure port and renderer)

- [ ] Compiler: the text properties of the grammar.
- [ ] `morok-platform`: Core Text shaping, line breaking, bidi and fallback fonts; the measure port
  with a cache keyed by string, style and width; system font resolution for the `font-` tokens.
- [ ] Atlases: the mask and color atlases with shelf-packed pages and page-level LRU eviction;
  glyphs rasterized by font, glyph, size, screen scale and subpixel phase; color glyphs in the color
  atlas; the glyph pipeline in the renderer.
- [ ] `morok-core`: CSS line layout — line box and baseline, `letter-spacing`, `text-align`,
  `white-space`, `text-overflow: ellipsis`, `text-decoration`, `font-variant-numeric` — and the
  glyph-run primitive of the display list.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok-css --test text_properties
```
- Text properties compile and their unsupported forms are diagnostics (exit 0).
```timeout=900s
cargo test -p morok-platform --test text_shaping
```
- Core Text behind the real port of `morok-core` measures and shapes wrapped, truncated and bidirectional text as expected (exit 0).
```timeout=900s
cargo test -p morok-platform --test atlas_eviction
```
- Pages fill by shelves, the least recently used page is evicted at the budget and its glyphs come back on demand (exit 0).
```timeout=900s
cargo test -p morok-core --test line_layout
```
- Line boxes, baselines, alignment, ellipsis and decoration positions match the CSS model (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and comment rules are clean (exit 0).

### Phase 6: teststand — Chrome etalons and the perceptual comparator (consumes phases 2-5: compiler, tree, renderer, text)

- [ ] `crates/teststand`, the Chrome side: the pinned build fetched and driven over the DevTools
  protocol; the harness page and the markup generated from the element tree with state marker
  classes; screenshots at scale 2 on a transparent background in both schemes; element boxes read
  from the page.
- [ ] The etalon store under `crates/teststand/etalons/` and the commands `etalons render <suite>`,
  `etalons verify <suite>`, `check <suite>`.
- [ ] The MOROK side and the comparator: the offscreen frame and the layout boxes of the same tree;
  the geometry gate and the color gate with the thresholds of Knowledge; a diff PNG per failing case
  under `target/teststand/`.
- [ ] Suite `primitives`: boxes, flex and grid layouts, text sizes, weights, leading, letter
  spacing, wrapping, ellipsis and tabular numbers, in both themes.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok-teststand --test comparator
```
- Every comparator case of Knowledge behaves as listed (exit 0).
```timeout=1800s
cargo run -p morok-teststand -- etalons verify primitives
```
- The pinned Chrome re-renders the suite identically to the committed etalons (exit 0).
```timeout=900s
cargo run -p morok-teststand -- check primitives
```
- The real compiler, tree, renderer and text match Chrome within the thresholds in both themes (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and comment rules are clean (exit 0).

### Phase 7: bench — budget benchmarks with failing gates (consumes phases 3-5: frame pipeline; order-only after phase 6)

- [ ] `crates/bench`: the harness — scripted scenes, warm-up, per-stage timings taken from the
  frame pipeline, the 95th percentile, budgets from `crates/bench/budgets.toml`, a counting
  allocator, a failing exit code.
- [ ] Scenario `frame` on the reference scene: every stage and the CPU total within budget, zero
  allocations per steady frame.
- [ ] Scenario `idle`: a window with content and no events for 10 s presents no frame.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok-bench --test budget_gate
```
- Every budget-gate case of Knowledge behaves as listed (exit 0).
```timeout=1200s
cargo run --release -p morok-bench -- frame
```
- The real pipeline holds the stage budgets, the 4 ms total and zero allocations on the reference scene (exit 0).
```timeout=1200s
cargo run --release -p morok-bench -- idle
```
- No frame is presented over 10 s without events (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and comment rules are clean (exit 0).

### Phase 8: paint-effects — transforms, group opacity, filters and blending (consumes phases 2-7: compiler, renderer, stand, bench)

- [ ] Compiler: `opacity`, `transform`, `transform-origin`, `filter`, `backdrop-filter`,
  `mix-blend-mode` with the function lists of the grammar.
- [ ] Renderer: a transform is a matrix on the subtree's instances and never touches layout; a
  subtree gets an offscreen pass only when group opacity, a filter, a backdrop filter or a blend
  mode is set; damage uses the transformed bounds.
- [ ] Suite `effects` in the stand.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok-css --test effect_properties
```
- Effect properties compile and functions outside the lists are diagnostics (exit 0).
```timeout=900s
cargo run -p morok-teststand -- check effects
```
- Transforms, group opacity, filters and blend modes match Chrome within the thresholds (exit 0).
```timeout=1200s
cargo run --release -p morok-bench -- frame
```
- A scene without effects still holds its budgets: effects cost nothing when unused (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and comment rules are clean (exit 0).

### Phase 9: icons-images — Icon, Image and Canvas (consumes phases 4-6: renderer, atlases, stand; order-only after phase 8)

- [ ] `Icon`: SVG rasterized through `resvg` per size and screen scale; single-color icons in the
  mask atlas tinted with `currentColor`, multi-color icons in the color atlas.
- [ ] `Image`: PNG, JPEG and HEIC decoded off the main thread by the system decoder; a placeholder
  until pixels arrive; small images in the color atlas, large ones as separate textures under a byte
  budget; `object-fit` in the compiler.
- [ ] `Canvas`: the application draws with the frame's encoder inside the node's rectangle; the kit
  keeps z-order and clipping.
- [ ] Suite `icons-images` in the stand.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok-platform --test image_pipeline
```
- Images of the three formats decode off the main thread, large textures are evicted at the byte budget, a `Canvas` draw stays inside its clip (exit 0).
```timeout=900s
cargo run -p morok-teststand -- check icons-images
```
- Icons at every size and images with every `object-fit` match Chrome on the real atlases (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and comment rules are clean (exit 0).

### Phase 10: animation — transitions, keyframes and springs (consumes phases 3-8: tree, frame loop, effects, stand, bench; order-only after phase 9)

- [ ] Compiler: `transition`, `animation`, `@keyframes`, `cubic-bezier()`, `steps()`, `linear`.
- [ ] `morok-core`: the timeline — interpolation of lengths, colors, shadows, opacity and
  transforms; keyframe animations with delay, iteration count, direction and fill mode; the
  critically damped spring used later by scrolling and gestures.
- [ ] The display link is held only while an animation or a spring runs. With "Reduce motion" on,
  transitions are instant, springs become short fades and infinite animations stop.
- [ ] Suite `motion` in the stand: transitions and keyframes sampled at fixed times on both sides.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok-core --test animation_timeline
```
- Interpolation, easing curves, keyframe options, the spring and reduced motion give the expected values over time (exit 0).
```timeout=900s
cargo run -p morok-teststand -- check motion
```
- Sampled transitions and keyframes on the real renderer match Chrome at the same times (exit 0).
```timeout=1200s
cargo run --release -p morok-bench -- idle
```
- After the last animation ends the display link stops and no frame is presented (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and comment rules are clean (exit 0).

### Phase 11: reactive — the signal graph (order-only after phase 10)

- [ ] `crates/reactive`: signals, memos, effects, ownership scopes with disposal, batching and
  untracked reads on an arena with ID handles.
- [ ] Synchronous `flush`: every effect runs once per flush with consistent values; an effect that
  writes a signal joins the same flush; a cycle is reported instead of looping.
- [ ] No allocation for a signal write followed by a flush once the graph is built.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok-reactive --test graph
```
- Dynamic dependencies, the diamond, disposal, batching, re-entrant writes and cycle detection behave as specified (exit 0).
```timeout=900s
cargo test -p morok-reactive --test steady_allocations
```
- A write and a flush on a built graph allocate nothing (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and comment rules are clean (exit 0).

### Phase 12: input-focus — pointer, keyboard and focus (consumes phases 3-4: tree and view; order-only after phase 11)

- [ ] `morok-core`: pointer events and hit testing through clips and transforms; `hover` and
  `active` flags with pointer capture.
- [ ] Focus: traversal in tree order, a `tabindex`-like attribute, the keyboard-or-pointer modality
  behind `focus-visible`, `focus-within`, focus traps; key events routed from the focused node
  upwards; the shortcut registry at window and component level.
- [ ] `morok-platform`: `NSEvent` mouse and key events translated into core events; first-responder
  handling.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok-core --test hit_testing
```
- Hits respect clips, rounded corners and transforms; hover, active and capture follow the pointer (exit 0).
```timeout=900s
cargo test -p morok-core --test focus_navigation
```
- Tab order, the tab-index attribute, focus-visible modality, focus-within, traps and shortcuts behave as specified (exit 0).
```timeout=900s
cargo test -p morok-platform --test event_translation
```
- Synthesized `NSEvent`s reach the real tree of `morok-core` as the expected events and flags (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and comment rules are clean (exit 0).

### Phase 13: accessibility — the AccessKit tree and the macOS adapter (consumes phase 12: focus and actions)

- [ ] `morok-core`: an AccessKit node for every node with a role — role, name, value, bounds,
  states, supported actions — and a tree-update delta produced in the frame of the display list.
- [ ] `morok-platform`: the adapter on the kit's view, updates raised on the main thread, actions
  (focus, default action, increment, decrement, set value) routed back as core events.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok-core --test accessibility_tree
```
- Roles, names, values, bounds and states mirror the tree and an update carries only what changed (exit 0).
```timeout=900s
cargo test -p morok-platform --test accessibility_adapter
```
- A real view answers accessibility queries through the adapter and a focus action moves the real focus of `morok-core` (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and comment rules are clean (exit 0).

### Phase 14: view-button — the component model, the facade and Button (consumes phases 11-13: signals, input, accessibility)

- [ ] `crates/components`, module `view`: a component is a function called once at mount; node
  builders; node properties bound to signals through effects; keyed `Show` and `For`; a scope lives
  as long as its subtree; direct access to lower-layer nodes for hot paths. Its `build.rs` compiles
  `design/system/tokens.css` and the component stylesheets.
- [ ] `crates/morok`: the composition root — the application owns the reactive runtime and the
  shared GPU resources, a surface owns one tree root, the frame runs in the stage order of
  Knowledge, several windows are several surfaces.
- [ ] `Button`: `button.css` with every state and the `primary` variant, activation by pointer and
  by Space and Return, the AccessKit role and name.
- [ ] Suite `button` in the stand; bench scenario `cold-start`.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok-components --test view_model
```
- Bindings, `Show`, `For` and scope disposal work on the real `morok-reactive` and `morok-core`, with no node rebuilt on a signal change (exit 0).
```timeout=900s
cargo test -p morok --test button_interaction
```
- In a real window a click, Space, Return and an accessibility action fire the handler; a disabled button does not (exit 0).
```timeout=900s
cargo run -p morok-teststand -- check button
```
- Every state and variant of `Button` matches Chrome in both themes (exit 0).
```timeout=1200s
cargo run --release -p morok-bench -- cold-start
```
- The first frame with content is presented within 150 ms of process start (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and comment rules are clean (exit 0).

### Phase 15: text-input — plain-text editing and the text fields (consumes phase 14: component model and surfaces)

- [ ] `morok-core`: the plain-text buffer — caret movement by grapheme, word, line and document
  with the macOS key bindings, selection, undo and redo, wrapping, scrolling the caret into view.
- [ ] `morok-platform`: `NSTextInputClient` on the view (marked text, the candidate-window
  rectangle, dictation) and the clipboard port over `NSPasteboard`.
- [ ] Compiler: `::placeholder`, `::selection`, `caret-color`.
- [ ] `TextField`, `TextArea`, `SearchField` with stylesheets in every state and AccessKit text
  roles, value and selection; suite `text-fields`; bench scenario `input-latency`.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok-core --test text_editing
```
- Caret movement, selection, undo grouping, wrapping and scrolling into view behave as on macOS (exit 0).
```timeout=900s
cargo test -p morok-platform --test text_input_client
```
- A marked-text sequence, its commit and a paste reach a real mounted `TextField` through the input client (exit 0).
```timeout=900s
cargo run -p morok-teststand -- check text-fields
```
- The three fields match Chrome in every state, with placeholder, selection and caret (exit 0).
```timeout=1200s
cargo run --release -p morok-bench -- input-latency
```
- The glyph of a keypress is in the frame of its event (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and comment rules are clean (exit 0).

### Phase 16: controls — the remaining controls (consumes phase 14: component model; order-only after phase 15)

- [ ] `IconButton`, `Checkbox`, `Radio` (group, arrow keys), `Switch`, `Slider` (drag and keys),
  `SegmentedControl`, `ProgressBar` (determinate and indeterminate), `Spinner` — each with its
  stylesheet in every state, AccessKit role, name and value, and keyboard behaviour.
- [ ] Suite `controls` in the stand.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok-components --test controls_behaviour
```
- Each control mounted through the real `view` model changes its signal by pointer, keyboard and accessibility action (exit 0).
```timeout=900s
cargo run -p morok-teststand -- check controls
```
- Every control matches Chrome in every state and both themes (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and comment rules are clean (exit 0).

### Phase 17: scroll — the Scroll element (consumes phases 10-12: spring and pointer input; order-only after phase 16)

- [ ] Compiler: `overflow: auto` and `scroll`, `position: sticky` inside a `Scroll`.
- [ ] `Scroll`: the offset moves the content as a transform without relayout; wheel and trackpad
  phases with system momentum; rubber banding on the spring; overlay scrollbars; keyboard scrolling;
  scrolling the focused node into view; AccessKit scroll actions.
- [ ] `morok-platform`: scroll events translated with their phases and precise deltas.
- [ ] Suite `scroll` in the stand.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok-core --test scrolling
```
- Offsets, limits, sticky positions and the rubber band on the real spring behave as specified, with no layout pass on scroll (exit 0).
```timeout=900s
cargo run -p morok-teststand -- check scroll
```
- Scrolled content, sticky elements and scrollbars match Chrome at fixed offsets (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and comment rules are clean (exit 0).

### Phase 18: list-tree-tabs — virtualization, List, Tree and Tabs (consumes phase 17: scroll offset and viewport)

- [ ] Virtualization: the visible range from the scroll offset and row heights; a pool of row
  subtrees reused while scrolling, so the node count depends on the viewport and never on the data.
- [ ] `List` (single and multiple selection, keyboard), `Tree` (expand, collapse, levels, arrow
  keys), `Tabs` — stylesheets in every state and AccessKit roles with level, expanded and selected.
- [ ] Suite `collections` in the stand.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok-components --test virtualization
```
- With 1,000,000 rows in a real `Scroll` the node count stays bounded by the viewport and rows are reused (exit 0).
```timeout=900s
cargo run -p morok-teststand -- check collections
```
- `List`, `Tree` and `Tabs` match Chrome in every state and both themes (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and comment rules are clean (exit 0).

### Phase 19: table — Table and the million-row budget (consumes phase 18: virtualization)

- [ ] `Table`: virtualized rows and columns, a header, column resizing, sorting with its indicator,
  column reordering, row selection, keyboard navigation; AccessKit table, row, cell and
  column-header roles with the sort state.
- [ ] Suite `table` in the stand; bench scenario `scroll-1m`.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok-components --test table_behaviour
```
- Resize, sort, reorder, selection and keyboard work on the real virtualization core (exit 0).
```timeout=900s
cargo run -p morok-teststand -- check table
```
- `Table` matches Chrome in every state and both themes (exit 0).
```timeout=1200s
cargo run --release -p morok-bench -- scroll-1m
```
- Every frame of a scripted scroll through 1,000,000 rows stays within the frame budget with zero allocations (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and comment rules are clean (exit 0).

### Phase 20: overlays — overlay surfaces and the overlay components (consumes phase 14: surfaces; order-only after phase 19)

- [ ] `morok-platform`: an overlay surface — a borderless window with its own Metal layer, placed
  against an anchor rectangle in screen coordinates, flipping at screen edges, free to extend past
  the main window; dismissal by outside click and Escape.
- [ ] `Popover`, `Tooltip`, `Menu` (keyboard, submenus), `ContextMenu` (`native` through `NSMenu`
  by default, `custom` as an overlay), `Dialog` and `Sheet` (focus trap, scrim, default and cancel
  keys), `Toast` — stylesheets and AccessKit roles.
- [ ] Suite `overlays` in the stand.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok-platform --test overlay_placement
```
- A real overlay window lands at the anchor, flips at a screen edge and extends past its parent (exit 0).
```timeout=900s
cargo test -p morok-components --test overlays_behaviour
```
- On real surfaces overlays open, trap and restore focus, and close by outside click and Escape (exit 0).
```timeout=900s
cargo run -p morok-teststand -- check overlays
```
- Every overlay matches Chrome in both themes (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and comment rules are clean (exit 0).

### Phase 21: select-combobox — Select and Combobox (consumes phases 15-20: text field, list, overlays)

- [ ] `Select` (overlay list, keyboard, type-ahead) and `Combobox` (a text field with a filtered
  list) — stylesheets in every state, AccessKit combobox and list-box roles.
- [ ] Suite `select-combobox` in the stand.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok-components --test select_combobox
```
- Both components choose a value by pointer, keyboard and typing on the real text field, list and overlay surface (exit 0).
```timeout=900s
cargo run -p morok-teststand -- check select-combobox
```
- Closed and open states match Chrome in both themes (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and comment rules are clean (exit 0).

### Phase 22: window-scaffold — window scaffolding and windows (consumes phase 14: surfaces and shortcuts; order-only after phase 21)

- [ ] `SplitView` (draggable dividers, limits, collapse), `Sidebar`, `Toolbar`, `StatusBar` —
  stylesheets, AccessKit roles, keyboard.
- [ ] Windows: opening and closing further windows from the application; opt-in blur behind a
  window through `NSVisualEffectView` under a transparent Metal layer; the menu bar built from the
  shortcut registry.
- [ ] Suite `scaffold` in the stand.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok-components --test scaffold_behaviour
```
- Dividers drag within their limits and collapse; sidebar, toolbar and status bar respond to keyboard and accessibility actions (exit 0).
```timeout=900s
cargo test -p morok --test multiple_windows
```
- Two real windows keep separate trees and focus, share GPU resources, and a registered shortcut appears in the menu bar (exit 0).
```timeout=900s
cargo run -p morok-teststand -- check scaffold
```
- The scaffolding components match Chrome in both themes (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and comment rules are clean (exit 0).

### Phase 23: drag-drop — drag and drop and the clipboard (consumes phase 12: pointer input; order-only after phase 22)

- [ ] `morok-core`: the drag session — source, typed payload, drop targets with their state flags,
  pointer-driven drags inside the application.
- [ ] `morok-platform`: system drag and drop of files between Finder and the application
  (`NSDraggingSource`, `NSDraggingDestination`), the drag image rendered from the dragged subtree;
  the clipboard port extended to files, URLs and custom types.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok-core --test drag_session
```
- A drag driven through the real pointer input enters, hovers and drops on a target with the expected flags and payload (exit 0).
```timeout=900s
cargo test -p morok-platform --test pasteboard_types
```
- Text, files, URLs and a custom type round-trip through a real named pasteboard (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and comment rules are clean (exit 0).

### Phase 24: wire — the catalog application on the main path (consumes phases 1-23)

- [ ] `crates/catalog`: the catalog application — a page per component family and the two screens
  of the design book (the two-pane file manager, the disk map with a `Canvas` sunburst), built on
  the published crates only, with its own stylesheets compiled by its `build.rs` and a second
  tokens file to switch to.
- [ ] The end-to-end test of the main path on real modules: launch, click, type, scroll the
  million-row table, open a popover and a menu, switch the theme and the tokens, open a second
  window; assertions on the tree, the AccessKit tree and the rendered frame.
- [ ] Suite `screens` in the stand; every bench scenario on catalog scenes.
- [ ] `docs/MOROK-ARCHITECTURE.md` brought in line with what was built: crate layout, style schema,
  surfaces, the stand pipeline.

**Done when (EXECUTABLE):**
```timeout=1800s
cargo test -p morok-catalog --test wire_main_path
```
- The catalog walks the main path on the real modules of every crate; nothing is mocked above the system boundary (exit 0).
```timeout=3600s
cargo run -p morok-teststand -- etalons verify all
```
- The pinned Chrome re-renders every suite identically to the committed etalons (exit 0).
```timeout=1800s
cargo run -p morok-teststand -- check all
```
- Every suite, the two screens included, matches Chrome within the thresholds (exit 0).
```timeout=1800s
cargo run --release -p morok-bench -- all
```
- Every budget holds on catalog scenes (exit 0).
```
grep -q morok-style docs/MOROK-ARCHITECTURE.md
```
- The architecture document names the crate layout that was built (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and comment rules are clean (exit 0).
