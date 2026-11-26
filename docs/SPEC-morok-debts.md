<!--
  SPEC-morok-debts.md — the debts of the kit after docs/SPEC-morok.md: behaviour that diverges from
  what the kit promises, the features the catalog lacks, rebinding rows without allocations, and
  the paths no test covers. 11 phases.

  DECISION BOUNDARY (fan of 2026-11-20, variant A chosen on the recommendation): one spec, the
  fixes before the features with an `until list-rebind` boundary between them; translations in
  double precision until the GPU; an image enters the application as encoded bytes and is shown
  by an `Image` component; the Finder contract is checked by machine, the gesture by hand.
  Rejected: B — two specs, the fixes first: a second wire phase over the same test, the manual
  Finder check without a home in the catalog until the second spec, and a second round of fan and
  review, for the order an `until` boundary already gives. C — one spec with a shifted origin of
  coordinates: a second coordinate convention under one scroller (laid-out content against the
  whole offset, origin-relative content against its local part), about ten readers of the offset
  from hit testing to text editing that must know which one they read, and a plain scroll view
  that needs a mode in which it never rebases; an error of mixing the two is silent.
-->

# Baseline

- Prior spec: `docs/SPEC-morok.md` — all 24 phases closed (run f46aa0ca), committed up to
  `66017cb` and pushed to `origin/main`; HEAD is `8d44a6f`.
- Prior spec-hash: sha256:f6f5944b422199331044d47267ea1d107ec63bb2ca1c854d562eb0bcf8417c24
- Exists and is not touched: `docs/GRILL-morok.md`, `design/morok-design-book.html`,
  `design/system/tokens.css`, `CLAUDE.md`; the dropdown of `Select` and `Combobox` stays on
  `VirtualRows` (`crates/components/src/option_list.rs`), not on `List`; a drop target shows its
  state through the hover flag (`crates/core/src/drag/drag_router.rs:69`), with no new
  pseudo-class.
- Already done at HEAD: the architecture says of toasts what the code does. Commit `8d44a6f`
  ("Fix where toasts, dialogs and sheets are drawn in the architecture") brought
  `docs/MOROK-ARCHITECTURE.md:75-79` and `:163-164` in line with
  `crates/components/src/toast_stack.rs:14-15`; there is no phase for it.
- Out of scope: `OptionList` (`option_list.rs:62-66`) and `Tree` (`tree.rs:84-88`) clone a label on
  every rebind — not asked; driving Finder from a test; publishing to crates.io.
- Version: no release automation was found (no `.github/`, no release script, `.cargo/config.toml`
  holds only the `xtask` alias) and nothing is published, so the workspace stays at `0.1.0` and
  there is no version-bump phase, although `scroll_offset`, `SplitPane::sized` and `List::new`
  change their signatures.
- Rationale of the choice — staged decision note (id after accept).

# Stack

- No new crates and no new third-party dependencies.
- `morok-core`: cancelling a drag, carrying a drag into another tree, the drag picture without the
  clips of ancestors; `Transform` with an `f64` translation, the scroll state and the displacement
  in `f64`; `PlacedOverlay` with the shift of its panel; a live setting of accessible elements;
  repainting a canvas on request.
- `morok-platform`: the closing of a window by its frame (`NSWindowWillCloseNotification`); the
  surface under a screen point for drags between surfaces; the shift of an overlay passed to its
  content; `TransformEntry` narrowing translations to `f32`.
- `morok-components`: `Surfaces` carries pointer drags and scroll input between surfaces; the arrow
  of `Popover`; `Combobox` (the chevron, the live node); `ScrollArea` and `VirtualRows` in `f64`;
  `List` with a written label; `SplitView` with an axis; the component `Image` (`image.rs`,
  `image.css`).
- `morok`: `Window::on_closed`, `Application::add_image`, `Application::image_status`, the
  re-export of `ImageStatus`; the dev-dependency `objc2-core-graphics` with `CGWindow` and
  `CGImage`; the test target `images` (`harness = false`).
- `morok-catalog`: the volumes and the Treemap of the disk map; the page `Images` with its own
  pictures in `crates/catalog/images/`; a closed window leaves the catalog.
- `morok-teststand`: a canvas drawn by the painter the case gives; rectangles among canvas shapes;
  new cases in `scaffold`, `icons-images` and `screens`.
- `morok-bench`: the scenario `scroll-list`.

# Conventions

- Every convention of `docs/SPEC-morok.md` holds unchanged: no heap allocation on the hot path of a
  frame; redraw by damage only; a component is `<name>.rs` beside `<name>.css` with all its states,
  an AccessKit node with a role and a name, keyboard navigation and a stand case; `unsafe` only in
  `morok-platform`; a test that touches AppKit is a `harness = false` target on the main thread;
  etalons come only from `etalons render`; the stand thresholds and the bench budgets do not
  change.
- Translations are `f64` until the GPU: the scroll offset, the displacement and the translation of
  a `Transform`. A translation becomes `f32` only in `TransformEntry`, after composition.
- A behaviour that changes updates, in the same phase, the test that pinned the old behaviour.
- Floats in tests are compared through geometry types or `to_bits`, never with a bare `assert_eq!`
  on `f32` or `f64` (`clippy::float_cmp` under `-D warnings`).
- Comments are English, rare, and never mention phases, tasks or this spec; `docs/` is English.

# Knowledge

- **Escape.** During a drag of the surface a key reaches the focused element as usual
  (`ElementTree::key_input`, `crates/core/src/element_tree.rs:938`). `DragOutcome::Cancelled`
  exists and `DragRouter::give_up` gives it. Escape with no modifiers, pressed while a drag of the
  surface is under way, ends it as `give_up` does — the target hears `Left`, the source
  `DragEnded(Cancelled)` — and reaches no shortcut, text edit or focus move. A drag handed to the
  system belongs to AppKit, whose tracking loop cancels it on Escape by itself.
- **Drag picture.** `SubtreePicture` (`crates/core/src/paint/subtree_picture.rs:31`, `:45-51`)
  copies every clip and group open around the subtree, so a scroll viewport or a clipping ancestor
  cuts the picture. The picture keeps the groups around the subtree, which place it, and drops the
  clips of its ancestors; clips inside the subtree stay. Its rectangle stays the union of the
  visual bounds of the subtree, cut to the surface it is drawn on.
- **Pressing selected text.** `begins_drag` returns before a drag when the press selects text in
  the focused text input (`crates/core/src/input/pointer_router.rs:87-90`, `text_drag`). No test
  covers it.
- **Drags between surfaces.** A drag of the surface lives in the `InputState` of one tree. AppKit
  keeps sending `mouseDragged:` and `mouseUp:` to the view the press began in, outside its window
  too, and `input_translation` passes the location unclamped. A drag begun in an overlay is
  stranded today once the pointer leaves the panel: `Surfaces::pointer_input` sends the moves to
  the parent while the overlay keeps the session (`crates/components/src/view/surfaces/overlays.rs:181-198`).
  The platform tells which surface of the application is topmost under a screen point
  (`windowNumberAtPoint:belowWindowWithWindowNumber:`, the surface view of that window) and where
  its origin lies against another surface's. `Surfaces` carries the drag into that surface through
  the entry a tree has for drags that began outside it (`system_drag_entered`, `_moved`, `_exited`,
  `_dropped` in `crates/core/src/element_tree/dragging.rs`). The session stays in the source tree,
  which ends it with the outcome the other surface gives and lets go of its own target while the
  pointer is elsewhere. An overlay is a surface like a window.
- **Closing by the frame.** No window delegate and no close notification exist. The button of the
  frame hides the `NSWindow` (`setReleasedWhenClosed(false)`, `crates/platform/src/window.rs:67`)
  while `morok::Window`, its `HostedSurface` and its surface live on, and `menu_bar::forget` runs
  only on drop. The surface view observes `NSWindowWillCloseNotification` as it observes the key
  notifications (`crates/platform/src/surface_view.rs:505-528`): its content hears that the window
  closed, its overlays go, its frame loop stops and the menu bar forgets its menus. The handler of
  `Window::on_closed` runs on the next turn of the run loop, so that it may drop the value; a
  window that closed closes nothing twice when dropped. The button of the frame sends
  `performClose:`, and the test sends it too.
- **Menu bar between windows.** The bar changes lazily: the first frame of the new key window
  calls `sync_menu_bar` (`crates/platform/src/surface_view.rs:771`, `:779-798`), so a test waits for
  that frame. No test switches the key window today.
- **Blur by its pixels.** `multiple_windows` checks only the hierarchy of views. The capture uses
  `CGWindowListCreateImage` (objc2-core-graphics 0.3.2, features `CGWindow` and `CGImage`,
  deprecated in favour of ScreenCaptureKit, which the workspace does not have) over the screen rect
  of two windows of the test itself: a backdrop of two vivid halves meeting at a one-pixel edge,
  and over it a window that asks for blur and draws nothing there. The blur shows as a transition
  across that edge several pixels wide whose ends lean to the two colours; a window that asks for
  no blur shows its own background there. Not verified: whether macOS 26 hands a process the
  pixels of its own windows without the Screen Recording permission. A red gate that names that
  permission is a question for the user, never a reason to weaken the test. The display stays
  awake for window gates (`caffeinate -d -u`).
- **The wheel around an overlay.** The pointer over the room an overlay leaves around its panel
  already goes to the surface under it (`overlays.rs:155-202`). The wheel is dropped there:
  `HostedSurface::scroll_input` (`crates/morok/src/hosted_surface.rs:137-142`) goes straight to the
  overlay's tree, which finds no scroller (`crates/core/src/element_tree/scrolling.rs:130-152`).
  Scroll input takes the same way through `Surfaces` as pointer input.
- **The arrow of the popover.** `OverlayPlacement::place` keeps the panel on the screen with
  `kept_inside` (`crates/core/src/overlay/overlay_placement.rs:29-41`, `:104-106`) and reports only
  the frame and the side. The arrow stands where its stylesheet puts it (`left: 28px`,
  `crates/components/src/popover.css:19-35`), so a shifted panel points it past the anchor.
  `PlacedOverlay` reports the shift, the platform passes it with the side, and the arrow moves
  against the shift from its place, kept on the straight part of the panel's edge.
- **Combobox and VoiceOver.** The field and the options lie in different AccessKit trees: the list
  is an overlay with its own window and adapter
  (`crates/platform/src/accessibility/surface_accessibility.rs:27-38`), so an active descendant of
  the field cannot name an option; the consumer maps focus to an active descendant inside one tree
  only (accesskit_consumer 0.39.2, `tree.rs:538-543`). accesskit_macos 0.27.2 announces a node whose
  live setting is not `Off` when its label changes (`event.rs:300-310`,
  `NSAccessibilityAnnouncementRequestedNotification`). The field's tree holds a node of live
  `Polite` labelled with the highlighted option and its place in the list ("Movies, 3 of 12"), with
  no label while nothing is highlighted.
- **The chevron.** The options shown are always filtered by the text of the field
  (`crates/components/src/combobox.rs:141-146`); the chevron only opens (`:161-168`), and
  `crates/components/tests/select_combobox.rs:826` pins that it shows "Movies" alone. Opened by the
  chevron, the list shows every option with the chosen one highlighted; the next edit of the text
  filters again. A field whose text matches nothing no longer opens an empty list on the chevron
  (`crates/components/src/dropdown.rs:53-54`).
- **Scroll precision.** Every coordinate is `f32`: geometry, Taffy, display items, Metal. Beyond
  2^24 = 16,777,216 px an `f32` has steps of 2 px. With a million rows of 26 or 28 px only the
  offset loses precision: row tops of an even height stay exact below 2^25, and the subtraction of
  two near values is exact. The offset enters at one place (`crates/core/src/paint/group_transform.rs:27-30`)
  and meets the displacement of a row in `Transform::then` (`ty = row_top − shown`). `ScrollState`,
  `ScrollArea`, `scroll_to`, `scroll_offset`, `scroll_limit`, the displacement and the translation
  of `Transform` become `f64`; the linear part stays `f32`. `TransformEntry`
  (`crates/platform/src/render/transform_entry.rs:33-43`) narrows the composed translation, which
  is small for anything on screen. The return spring animates the distance past the limit, not the
  offset, and `carried` goes. Content laid out by Taffy beyond 2^24 keeps the precision of `f32`
  layout; virtualized content is where such lengths occur. An offset no `f32` holds (an odd number
  past 2^24) is what tells the two apart in a test.
- **List without allocations.** `List` builds a `String` per rebound row
  (`crates/components/src/list.rs:26`, `:77`). `Table` already writes into a buffer each cell keeps
  (`crates/components/src/table.rs:30`, `body_cell`), and `ElementTree::set_text` reuses the text it
  holds (`crates/core/src/element_tree.rs:403-415`). `List::new` takes
  `label_of: impl Fn(usize, &mut String) + 'static`, and each slot keeps one buffer for its text and
  its accessible name. The bench counts the heap allocations of a frame with a global allocator
  (`crates/bench/src/counting_allocator.rs`) against `allocations = 0`. `scroll-list` scrolls the
  list "Documents" of the Collections page (10,000 rows of 28 px) from its first row to its last
  over the frames of the budget, so every slot is rebound every frame, against the budgets
  `scroll-1m` meets.
- **Vertical split.** The axis is fixed in `split_view.css` (a row, `width`), `split_divider.rs`
  (`position.x`, ArrowLeft and ArrowRight) and `split_pane.rs` (`width`). `SplitView::axis` takes
  `SplitAxis::Horizontal` (the default) or `SplitAxis::Vertical`, and `SplitPane::sized` takes a
  length along the axis. A vertical view stacks its panes; its dividers are horizontal lines moved
  by ArrowUp and ArrowDown, and its `Splitter` nodes carry the orientation of their line. The book
  draws only the side-by-side split (`design/morok-design-book.html:807`, `:346-347`); the vertical
  one is its geometry turned by a quarter.
- **Image API.** The renderer decodes encoded bytes off the main thread
  (`Renderer::add_image(Vec<u8>) -> ImageId` and `image_status`,
  `crates/platform/src/render/renderer.rs:153-160`), and the tree shows an `ImageId` at a natural
  size (`ElementTree::set_image`, `crates/core/src/element_tree.rs:464`); neither reaches an
  application. `Application::add_image(encoded: Vec<u8>) -> ImageId`,
  `Application::image_status(ImageId) -> ImageStatus`, and
  `Image::new(image: impl Into<Property<ImageId>>, label: impl Into<Property<String>>)`, fitted by
  the `object-fit` of its class. While decoding it is a box of its CSS size; once decoded the host
  gives it its natural size; when unreadable the box and its name stay. Bytes are the one way in:
  an application reads files itself (`include_bytes!`, `std::fs::read`, a drop from Finder).
  Rejected: an image the component registers on mount from a path or bytes — the same picture
  mounted twice would decode twice, reading files and their errors would enter the components,
  which depend only on ports, and there would be two ways.
- **Disk map.** The `place` signal reaches only the sidebar (`crates/catalog/src/disk_map.rs:86`,
  `:99`); the chart, the readouts and the description are fixed to Macintosh HD (`:159-200`) and
  the data is the constant `FOLDERS` (`:18-55`). A canvas cannot be repainted on request: `set_canvas`
  with the same id changes nothing (`crates/core/src/element_tree.rs:495-508`), and a frame without
  damage presents nothing (`crates/platform/src/surface_view.rs:751`). `ElementTree` repaints a
  canvas on request by damaging its box. The book draws the Treemap and Sunburst switch in the
  toolbar of the disk map (`design/morok-design-book.html:840`). The Treemap is a squarified layout
  of the folders over the chart, each folder's rectangle shared by its children, in the colours and
  opacities of the sunburst, filled through `CanvasEncoder::fill`.
- **The painter in the stand.** The disk-map case draws through `CanvasShapesPainter`
  (`crates/teststand/src/suites/screens.rs:50-62`), a copy of the logic of `SunburstPainter`, so the
  painter of the catalog is checked by one pixel in `wire_main_path` alone. `Scene` takes, for a
  canvas, the shapes Chrome draws and the painter MOROK draws with: a case about the canvas itself
  passes `CanvasShapesPainter` over its shapes, the disk-map cases pass the painter of the catalog,
  and the comparator holds that painter against an independent drawing of the same shapes.
- **Finder by machine and by hand.** By machine, out: a real `NSDraggingSession` from a component
  carrying files; the drag pasteboard (`NSPasteboardNameDrag`) holds one `public.file-url` per
  file, and the operation mask of the source for `NSDraggingContextOutsideApplication` holds Copy.
  By machine, in: a `StagedDrag` (`crates/platform/tests/system_drag.rs:210-267`) with the mask
  Finder offers (every operation) and a unique pasteboard of real temporary files and a folder; the
  component receives every path and the view answers Copy. By hand: the gesture itself, both ways,
  on the Images page of the catalog. Rejected: driving Finder from a test (AppleScript, `CGEvent`)
  — it needs the Accessibility and Automation permissions and depends on where Finder puts its
  windows, so a red gate would say nothing about the kit.
- **Limits of the gates.** The gates prove trees, AccessKit trees, offscreen frames, the captured
  pixels of the test's own windows and allocation counts. Human acceptance at the `wire` boundary:
  VoiceOver speaks the highlighted option of a combobox under the arrows; an image dragged from
  Finder onto the Images page shows, and dragged out onto the Desktop is copied there; the blur
  behind a window looks like the system's; a drag between the two windows of the catalog by hand;
  a popover opened at the right edge of the screen points at its button.

# Gameplan

<!-- 11 phases, a chain: each phase depends on the one before it. "consumes phase N" is real code
     consumption and comes with a criterion on the real module; "order-only after phase N" is
     ordering alone. Phases 1-7 are the fixes, 8-10 the features; `until list-rebind` closes the
     fixes. `cargo xtask check` closes every code phase. -->

### Phase 1: drag-core — Escape, the drag picture and pressing selected text

- [ ] `morok-core`: Escape with no modifiers, pressed while a drag of the surface is under way,
  cancels it — the target hears `Left`, the source `DragEnded(Cancelled)` — and reaches no
  shortcut, text edit or focus move.
- [ ] `SubtreePicture`: the picture keeps the groups around the subtree and drops the clips of its
  ancestors; clips inside the subtree stay.
- [ ] Tests in `crates/core/tests/drag_session.rs`: Escape cancels a drag and the next press
  drags afresh; the picture of a source inside a clipping, scrolled ancestor holds the part the
  ancestor's clip hides; a press in the selected text of the focused text input inside a source
  selects text past the slop and starts no drag.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok-core --test drag_session
```
- Escape cancels a drag of the surface, the picture holds what an ancestor clips, and a press in selected text drags nothing (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and comment rules are clean (exit 0).

### Phase 2: drag-across — drags between surfaces, the drag picture by its pixels and Finder by machine (consumes phase 1: cancelling a drag)

- [ ] `morok-platform`: the surface of the application topmost under a screen point and its origin
  against another surface's, for a view whose press goes on outside it.
- [ ] `morok-core`: the entry of a tree for drags that began outside it takes a drag of another
  surface of the application with its payload; the source tree ends its session with the outcome
  the other surface gives.
- [ ] `morok-components`: `Surfaces` carries a pointer drag across the surfaces of the
  application, windows and overlays alike: over another surface the drag enters, moves over and
  drops on its targets in that surface's coordinates while the source tree lets go of its own
  target; a release over nothing of the application cancels it; Escape cancels it wherever it is.
- [ ] Tests: `crates/core/tests/drag_session.rs` — a tree takes a drag of another surface and its
  drop ends the source's session; `crates/components/tests/drag_and_drop.rs` — a drag from one
  surface drops on a target of another and the source hears `Dropped`, a drag leaves an overlay's
  panel for its parent, Escape cancels a drag over another surface and that surface's target hears
  `Left`; `crates/morok/tests/drag_windows.rs` — two real windows, events through `sendEvent`, a
  drag from the first drops on a target of the second.
- [ ] Tests in `crates/platform/tests/system_drag.rs`: the pixels of the `NSImage` the view hands
  to the dragging session — the centre of the dragged card has its colour, its top row is the
  card's top, the part an ancestor clips is drawn; out to Finder — a real session from a source
  carrying two files: the drag pasteboard holds one `public.file-url` per file and the source's
  mask outside the application holds Copy; in from Finder — the mask Finder offers and a pasteboard
  of real temporary files and a folder: the component receives every path and the view answers
  Copy.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok-components --test drag_and_drop
```
- A drag crosses surfaces and drops on a target of another one, and Escape cancels it there through the real cancel of morok-core (exit 0).
```timeout=900s
cargo test -p morok-core --test drag_session
```
- A tree takes a drag of another surface and its drop ends the session of the source (exit 0).
```timeout=900s
cargo test -p morok --test drag_windows
```
- A drag goes from one real window to a target in another (exit 0).
```timeout=900s
cargo test -p morok-platform --test system_drag
```
- The drag picture holds the right pixels the right way up, and the Finder contract holds both ways (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and comment rules are clean (exit 0).

### Phase 3: windows — closing by the frame, the menu bar between windows and the blur by its pixels (order-only after phase 2)

- [ ] `morok-platform`: the surface view observes `NSWindowWillCloseNotification`; a window closed
  by the button of its frame tells its content, drops its overlays, stops its frame loop and lets
  the menu bar forget its menus.
- [ ] `morok`: `Window::on_closed(&self, handler: impl FnOnce() + 'static)` — the handler runs on
  the next turn of the run loop after the frame closed the window; a window that closed closes
  nothing twice when dropped.
- [ ] `morok-catalog`: a window closed by its frame leaves `Catalog.windows`
  (`crates/catalog/src/catalog.rs:71`, `:142`).
- [ ] `crates/morok/Cargo.toml`: the dev-dependency `objc2-core-graphics` with `CGWindow` and
  `CGImage`.
- [ ] Tests in `crates/morok/tests/multiple_windows.rs`: `performClose:` on the further window —
  the handler hears it, the window is off screen, the menu bar shows the menus of the first window
  once that one is key, and dropping the value afterwards changes nothing; the keyboard moves
  between the windows (`makeKeyWindow`) and after the next frame the menu bar shows the menus of
  the key window each time; the blur behind a window by its captured pixels, as Knowledge
  describes.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok --test multiple_windows
```
- Closing by the frame reaches the application, the menu bar follows the key window, and the blur shows in the captured pixels (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and comment rules are clean (exit 0).

### Phase 4: overlays — the wheel around an overlay and the arrow of the popover (order-only after phase 3)

- [ ] `morok-components`: scroll input over the room an overlay leaves around its panel goes to
  the surface under the overlay, in that surface's coordinates, as pointer input does;
  `HostedSurface::scroll_input` goes through `Surfaces`.
- [ ] `morok-core`: `PlacedOverlay` reports the shift of the panel along its side; unit tests in
  `crates/core/src/overlay/overlay_placement.rs` for a shift at each edge and none inside.
- [ ] `morok-platform`: the shift reaches the content of the overlay with its side.
- [ ] `morok-components`: the arrow of `Popover` moves against the shift from where its stylesheet
  puts it, kept on the straight part of the panel's edge.
- [ ] Tests: `crates/components/tests/overlays_behaviour.rs` — the wheel over the room of a popover
  scrolls the scroll view under it, and a shift moves the arrow against it within the edge;
  `crates/platform/tests/overlay_placement.rs` — a panel the right edge of the screen pushes
  reports its shift; `crates/morok/tests/overlay_windows.rs` — a popover opened by a button at the
  right edge of the screen points its arrow at the button.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok-core --lib overlay_placement
```
- The placement reports how far an edge pushed the panel, and nothing inside the bounds (exit 0).
```timeout=900s
cargo test -p morok-platform --test overlay_placement
```
- A real overlay window pushed by the screen edge hands its shift to its content (exit 0).
```timeout=900s
cargo test -p morok-components --test overlays_behaviour
```
- The wheel around a panel scrolls what lies under it, and the arrow follows the shift (exit 0).
```timeout=900s
cargo test -p morok --test overlay_windows
```
- A popover at the right edge of the screen points its arrow at its button (exit 0).
```timeout=900s
cargo run -p morok-teststand -- check overlays
```
- An unshifted popover still matches Chrome in both themes (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and comment rules are clean (exit 0).

### Phase 5: combobox — the chevron and the voice of the highlighted option (order-only after phase 4)

- [ ] `morok-components`: opened by the chevron, the list of `Combobox` shows every option with the
  chosen one highlighted; the next edit of the field filters again; the test at
  `crates/components/tests/select_combobox.rs:826` follows the new behaviour.
- [ ] `morok-core`: an element carries a live setting that the accessibility pass writes to its
  node; `morok-components`: `ElementBuilder::accessible_live`.
- [ ] `morok-components`: the field's tree holds a node of live `Polite` labelled with the
  highlighted option and its place in the list, with no label while nothing is highlighted.
- [ ] Tests: `crates/core/tests/accessibility_tree.rs` — the live setting of an element reaches its
  node; `select_combobox.rs` — the chevron over a field holding "Movies" shows every option,
  ArrowDown and ArrowUp change the label of the live node in the field's AccessKit tree to the
  highlighted option, and closing the list clears it.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok-core --test accessibility_tree
```
- The live setting of an element reaches its AccessKit node (exit 0).
```timeout=900s
cargo test -p morok-components --test select_combobox
```
- The chevron shows every option and the live node names the highlighted one (exit 0).
```timeout=900s
cargo run -p morok-teststand -- check select-combobox
```
- Select and Combobox still match Chrome in both themes (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and comment rules are clean (exit 0).

### Phase 6: scroll-precision — translations in double precision (order-only after phase 5)

- [ ] `morok-core`: `Transform` keeps its translation in `f64` and its linear part in `f32`;
  `ScrollState` (`offset`, `unresisted`, `shown`, `requested`, `limit`), `scroll_offset`,
  `scroll_to`, `scroll_limit` and the displacement of an element are `f64`; the return spring
  animates the distance past the limit; `carried` goes.
- [ ] `morok-platform`: `TransformEntry` narrows the composed translation to `f32`.
- [ ] `morok-components`: `ScrollArea.offset`, `VirtualRows` (row tops, the window of rows,
  `keep_in_view`) and `table_columns` are `f64`.
- [ ] Tests: `crates/core/tests/scrolling.rs` — scrolled to an odd offset past 2^24, the offset is
  exactly that and every one-pixel scroll moves the content by one pixel on the surface, and
  `small_scrolls_far_down_a_long_content_add_up` holds without a carry;
  `crates/components/tests/virtualization.rs` — a million rows of 28 px at an odd offset past 2^24:
  a one-pixel scroll moves the rows in view by one pixel; `crates/components/tests/table_behaviour.rs`
  — the million-row table at an odd offset past 2^24 keeps its cells exactly in place.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok-core --test scrolling
```
- An offset no f32 holds is kept, and one-pixel scrolls move the content by one pixel far down (exit 0).
```timeout=900s
cargo test -p morok-components --test virtualization
```
- Rows past 2^24 move by one pixel for a one-pixel scroll (exit 0).
```timeout=900s
cargo test -p morok-components --test table_behaviour
```
- The million-row table keeps its cells in place past 2^24 (exit 0).
```timeout=1800s
cargo run --release -p morok-bench -- scroll-1m
```
- Scrolling a million rows still meets every budget (exit 0).
```timeout=900s
cargo run -p morok-teststand -- check scroll
```
- Scroll views still match Chrome in both themes (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and comment rules are clean (exit 0).

### Phase 7: list-rebind — List rebinds rows without allocations (consumes phase 6: VirtualRows in f64)

- [ ] `morok-components`: `List::new(count, label_of: impl Fn(usize, &mut String) + 'static,
  selected)`; each row slot keeps one buffer, cleared and written on rebind, and sets its text and
  its accessible name from it.
- [ ] `morok-catalog`: the lists of the Collections page write their labels with `write!`.
- [ ] `morok-bench`: the scenario `scroll-list` in `crates/bench/src/scenarios.rs` — the list
  "Documents" of the Collections page scrolled by the wheel from its first row to its last over the
  frames of the budget, on a surface off screen, against the budgets `scroll-1m` meets.
- [ ] Tests in `crates/components/tests/virtualization.rs` follow the new `List::new`; a scroll of
  one row rebinds one slot to the text and the name of the row that came into view.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok-components --test virtualization
```
- The list on the real VirtualRows of double-precision offsets rebinds its slots to the rows in view (exit 0).
```timeout=1800s
cargo run --release -p morok-bench -- scroll-list
```
- Scrolling the list rebinds every slot every frame with no heap allocation and within the stage budgets (exit 0).
```timeout=900s
cargo run -p morok-teststand -- check collections
```
- Lists, trees and tabs still match Chrome in both themes (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and comment rules are clean (exit 0).

### Phase 8: split-axis — SplitView stacks panes vertically (order-only after phase 7)

- [ ] `morok-components`: `SplitView::axis(SplitAxis)` with `SplitAxis::Horizontal` (the default)
  and `SplitAxis::Vertical`; `SplitPane::sized(length: Signal<f32>, limits, content)`; a vertical
  view stacks its panes, its dividers are horizontal lines dragged along y, moved by ArrowUp and
  ArrowDown and folded as before; `Splitter` nodes carry the orientation of their line;
  `split_view.css` holds the rules of the vertical axis.
- [ ] Every caller of `SplitPane::sized` follows; the scaffolding page of the catalog shows a
  vertical split.
- [ ] Suite `scaffold`: a case `split-vertical`, its etalons rendered by `etalons render scaffold`.
- [ ] Tests in `crates/components/tests/scaffold_behaviour.rs`: a vertical view stacks its panes; a
  dragged divider changes the height of its pane within the limits; ArrowDown and ArrowUp move a
  focused divider; a drag far past the narrowest height folds a pane that can fold; the
  orientation of the splitter's node.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok-components --test scaffold_behaviour
```
- Horizontal and vertical views drag, step and fold their dividers within the limits (exit 0).
```timeout=900s
cargo run -p morok-teststand -- check scaffold
```
- The scaffolding components, the vertical split among them, match Chrome in both themes (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and comment rules are clean (exit 0).

### Phase 9: image — the image of the facade and the Images page (order-only after phase 8)

- [ ] `morok`: `Application::add_image(encoded: Vec<u8>) -> ImageId`,
  `Application::image_status(ImageId) -> ImageStatus` and the re-export of `ImageStatus`; the host
  gives an image its natural size once decoded and asks for frames while an image it shows is
  decoding.
- [ ] `morok-components`: `Image` (`image.rs`, `image.css`) — `Image::new(image, label)`, an element
  of role `Image` with its name, fitted by the `object-fit` of its class, a box of its CSS size
  while decoding and when unreadable.
- [ ] Suite `icons-images`: a case `image-component` — the component decoded under `cover` and
  `contain`, and unreadable; its etalons rendered by `etalons render icons-images`.
- [ ] `morok-catalog`: the page `Images` among the families — its own PNG and JPEG
  (`crates/catalog/images/`) under each `object-fit`, an unreadable image, and a drop zone where an
  image file dropped from Finder is read and shown and from which a shown picture is dragged out
  as its file.
- [ ] `crates/morok/tests/images.rs` (`[[test]] name = "images"`, `harness = false`): a PNG, a JPEG
  and unreadable bytes added to a real application; the status of each; the component at its
  natural size once decoded, the pixels of the picture in the frame image of the window, the box
  and the name of the unreadable one.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok --test images
```
- An application adds images, reads their status, and the component shows a decoded one at its natural size in the frame (exit 0).
```timeout=900s
cargo run -p morok-teststand -- check icons-images
```
- Icons, images and the Image component match Chrome in both themes (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and comment rules are clean (exit 0).

### Phase 10: disk-map — volumes, the Treemap and the painter in the stand (order-only after phase 9)

- [ ] `morok-core`: `ElementTree` repaints a canvas on request by damaging its box; a test in
  `crates/core/tests/damage_display_list.rs`.
- [ ] `morok-catalog`: every volume has its folders, its use and its path; choosing a volume
  rebuilds the chart, the readouts, the description and the accessible name; a segmented control
  Sunburst and Treemap in the toolbar of the disk map as the book draws it; the Treemap layout in
  `crates/catalog/src/treemap.rs`; the painter of the catalog draws the chosen volume in the chosen
  view, and the page repaints the canvas when either changes.
- [ ] `morok-teststand`: `Scene` takes for a canvas the shapes Chrome draws and the painter MOROK
  draws with; `CanvasShape` holds a rectangle as well as a ring sector; the disk-map cases draw
  with the painter of the catalog; new cases `disk-map-archive` and `disk-map-treemap` in
  `screens`, their etalons rendered by `etalons render screens`.
- [ ] Tests: `crates/catalog/tests/treemap.rs` — the rectangles tile the chart without overlap, the
  area of a folder is its share of the used space, its children lie inside it, colours and
  opacities as in the sunburst; `crates/catalog/tests/sunburst.rs` — the sunburst of each volume.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok-core --test damage_display_list
```
- A canvas repainted on request damages its box and presents (exit 0).
```timeout=900s
cargo test -p morok-catalog --test treemap
```
- The Treemap tiles the chart in proportion to the folders and their children (exit 0).
```timeout=900s
cargo test -p morok-catalog --test sunburst
```
- Each volume has its sunburst (exit 0).
```timeout=900s
cargo run -p morok-teststand -- check screens
```
- The screens match Chrome in both themes, the disk map drawn by the painter of the catalog in both views and for both volumes (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and comment rules are clean (exit 0).

### Phase 11: wire — the catalog on the main path of the debts (consumes phases 1-10)

- [ ] `crates/catalog/tests/wire_main_path.rs` on the real modules: a picture of the Images page
  dragged by the pointer from the first window onto the drop zone of the second, and a drag
  cancelled by Escape; the second window closed by its frame leaves the catalog and the menu bar
  shows the menus of the first; the wheel deep in the million rows at an odd offset past 2^24 moves
  the rows by one pixel; the chevron of a combobox shows every option and the live node names the
  highlighted one; the Images page shows a decoded picture in its pixels; choosing Archive changes
  the pixel and the description of the disk map, and Treemap shows rectangles; the scaffolding page
  shows its vertical split.
- [ ] `docs/MOROK-ARCHITECTURE.md` names the image of the facade (`add_image`), closing by the frame
  (`on_closed`) and translations in `f64`.

**Done when (EXECUTABLE):**
```timeout=1800s
cargo test -p morok-catalog --test wire_main_path
```
- The catalog walks the main path of the debts on the real modules of every crate; nothing is mocked above the system boundary (exit 0).
```timeout=3600s
cargo run -p morok-teststand -- etalons verify all
```
- The pinned Chrome re-renders every suite identically to the committed etalons (exit 0).
```timeout=1800s
cargo run -p morok-teststand -- check all
```
- Every suite matches Chrome within the thresholds (exit 0).
```timeout=1800s
cargo run --release -p morok-bench -- all
```
- Every budget holds, scroll-list among them (exit 0).
```
grep -q add_image docs/MOROK-ARCHITECTURE.md
```
- The architecture names the image of the facade (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and comment rules are clean (exit 0).
