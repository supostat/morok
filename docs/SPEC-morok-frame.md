<!--
  SPEC-morok-frame.md — one owner for the stages of a frame: a tree runs its stages in one place,
  the surfaces run theirs in one place, and the host, the bench, the stand and the test stages
  call those runs instead of copies of their order. 6 phases.

  DECISION BOUNDARY (fan of 2026-11-24, option A chosen on the recommendation): two runs nested in
  each other, each the one owner of its stages — `ElementTree::run_frame` in morok-core for the
  stages of a tree, `Surfaces::run_frame` in `morok_components::view` for the stages of a surface —
  and the stages themselves private to their crates, so the compiler holds the single owner.
  Rejected: B — one run in `components::view` only: the stages of the tree stay public because
  components call them, so the single owner is a convention; the tests of morok-core cannot reach
  components and keep a copy; scenes built as bare trees carry a runtime and a flush they do not
  need. C — the run in the facade: the tests of morok-components and morok-platform cannot depend
  on `morok`, which depends on them (a dev-dependency cycle builds a second copy of the crate whose
  types do not match), so their copies stay, and the order would sit above its stages.
-->

# Baseline

- Prior spec: `docs/SPEC-morok-rebind.md` — its one phase closed (run 9241ca2b); HEAD is `bcd3572`.
- Prior spec-hash: sha256:f2fc38225884387828668aea67535ba7e2bf9e3b6c5ba44216d7651ae734bae2
- The canonical order exists only in the private `HostedSurface::frame`
  (`crates/morok/src/hosted_surface.rs:102-114`): `refresh_hover` → `follow_scrolls` →
  `run_due_timers` → flush → `resolve_styles` → `size_images` → `compute_layout` →
  `build_display_list`. Copies of the order, each missing stages:
  `crates/bench/src/surface_pipeline.rs:84-93` (no hover, timers, pictures);
  `crates/bench/src/frame_pipeline.rs:62-66` and `crates/bench/src/scene_content.rs:45-47` (no
  pictures); `crates/teststand/src/scene.rs:269-277` (no hover, timers; pictures sized once before
  the frames in `crates/teststand/src/morok_side.rs:54`); `frame_of` in
  `crates/components/tests/support/mod.rs:271` (no hover, pictures);
  `crates/platform/tests/support/mod.rs:102-107` (no pictures);
  `crates/platform/tests/text_input_client.rs:100` (no scrolls, timers, pictures);
  `crates/core/tests/support/mod.rs:327` and the single-stage calls of
  `crates/core/tests/image_sizing.rs:47-56,133-137` and `crates/core/tests/frame_requests.rs:170`;
  the preliminary layout of a new overlay, `crates/morok/src/hosted_surface.rs:318-320` (no pictures).
- Exists and is not touched: what each stage does; `FrameRequests` and the port `FrameScheduler`;
  the frame loop of the platform; the placement of overlay windows (`HostedSurface::overlays` past
  its preliminary layout, `Surfaces::overlay_geometry`, `Surfaces::overlay_placed`) and the
  placement the stage of the components tests emulates (`frame_overlays_of`);
  `Surfaces::follow_overlay_anchors`; `ElementTree::refresh_hover`, the input step the tests of
  core and platform call on bare trees; the budgets in `crates/bench/budgets.toml`, the thresholds
  of the stand and the etalons.
- Out of scope: the frame loop of the platform dispatches a new block for every delayed frame
  request (`crates/platform/src/frame_loop.rs:41-57`), and the tree asks again in every frame while
  a caret blinks or a timer waits (`crates/core/src/frame_requests.rs:47-59`): a window-level task —
  the offscreen bench has no frame loop to dispatch (Knowledge, "The bench and the scheduler").
- Version: no release automation (no `.github/`, `.cargo/config.toml` holds only the `xtask`
  alias) and nothing is published: the workspace stays at `0.1.0`, with no version phase, although
  the stages of the tree and of the surface stop being public.
- Rationale of the choice — staged decision note (id after accept).

# Stack

- No new crates and no new dependencies.
- `morok-core`: module `frame` — `FrameStage`, `StageObserver<Stage>` with `Unobserved`,
  `TreeFrame`, `FrameOutcome`; the port `ImageStatuses` in `media`; `ElementTree::lay_out` and
  `ElementTree::run_frame`; the test target `frame_run`.
- `morok-components`: `SurfaceStage`, `SurfaceFrame`, `Surfaces::run_frame`; the test target
  `surface_frame`.
- `morok-platform`: `Renderer` implements `ImageStatuses`.
- `morok`: `HostedSurface` frames through `Surfaces::run_frame` and lays a new overlay out through
  `ElementTree::lay_out`.
- `morok-bench`: `StageClock`; `FrameSample` with time and allocations per stage; the pipelines
  and `SceneContent` on the runs.
- `morok-teststand`: `Scene::frame` on `Surfaces::run_frame` with the renderer as the port.
- `morok-catalog`: the test target `wire_frame_run` (`harness = false`).

# Conventions

- Every convention of `docs/SPEC-morok.md`, `docs/SPEC-morok-debts.md` and
  `docs/SPEC-morok-rebind.md` holds: no heap allocation on the hot path of a frame; etalons come
  only from `etalons render`; the stand thresholds and the bench budgets do not change.
- A stage is written in exactly one run. Nothing outside morok-core calls a stage of the tree,
  nothing outside morok-components calls a stage of the surface; from the wire phase on the
  compiler holds it.
- A new stage is added to its run and to its stage enum in the order it runs; no caller changes.
- A test that changes when its frame gets the stages its copy lacked is brought to what the kit
  does; a failing bench scene or stand case is a defect of the kit, never a reason to loosen a
  budget, a threshold or an etalon.
- A behaviour that changes updates, in the same phase, the test that pinned the old behaviour.
- Comments are English, rare, and never mention phases, tasks or this spec; `docs/` is English.

# Knowledge

- **The canonical order and why.** `refresh_hover` comes first: a resting pointer is hovered again
  over the boxes the last frame moved (`crates/components/src/view/surface.rs:212-213`).
  `follow_scrolls` and `run_due_timers` come before the flush: they write signals and run the
  handlers whose effects the flush runs (`surface.rs:87-88`, `:222-224`). The flush may close the
  surface (`crates/morok/src/hosted_surface.rs:106-108`). `size_images` sits between
  `resolve_styles` and `compute_layout`: `FrameRequests::frame_began` runs in `resolve_styles`, so a
  picture still decoding asks for the next frame only after it
  (`crates/core/src/element_tree.rs:494-498`).
- **Layers.** morok-core depends on morok-style alone and knows no signal graph, so the flush and
  the timers of components cannot be stages of core; morok-platform depends on core, not on
  components; morok-teststand depends on components, core, platform and reactive, not on the
  facade; morok-bench depends on the facade and platform. A test of a crate cannot depend on a
  crate that depends on it.
- **The run of a tree.** `TreeFrame<'a>` carries the viewport, the scale, the time of the frame,
  `&'a mut dyn TextShaper` and `&'a dyn ImageStatuses`. `ElementTree::lay_out(frame, observer)`
  runs `resolve_styles` → `size_images` (asking the port) → `compute_layout`;
  `ElementTree::run_frame(frame, observer)` is `lay_out` followed by `build_display_list`, so the
  order of the stages up to layout is written once. Both return `FrameOutcome` — the styled, the
  laid out and the rebuilt elements, as the stages count them today; `lay_out` rebuilds none.
  `FrameStage` is `Styles`, `Images`, `Layout`, `DisplayList`, declared in that order and `Ord`.
- **The run of a surface.** `Surfaces::run_frame(runtime, surface, frame, observer)` returns
  `Result<SurfaceFrame, FlushCycle>` (`crates/reactive/src/flush_cycle.rs`): `refresh_hover` →
  `follow_scrolls` → `run_due_timers` → flush; `SurfaceFrame::Closed` when the flush closed the
  surface, otherwise `SurfaceFrame::Ran(outcome)` after the run of its tree. `SurfaceStage` is
  `Hover`, `Scrolls`, `Timers`, `Effects`, `Tree(FrameStage)`, declared in that order and `Ord`; an
  adapter inside the run hands the stages of the tree to the observer of the surface as
  `SurfaceStage::Tree(stage)`. The surface is open when the run is called, as the host and the
  stage of the components tests check today.
- **Observing.** `StageObserver<Stage>` has `stage_ended(&mut self, stage: Stage)`; `Unobserved`
  implements it for every stage type and does nothing. The host passes `Unobserved`: one call
  through a vtable per stage, no allocation.
- **Bench buckets.** `crates/bench/budgets.toml` names four stages: events, layout, display_list,
  encoding. `StageClock` starts before the input or the change of the frame and, at every stage
  end, puts the time and the allocations since the last boundary into the budget stage of the
  run's stage by position: every stage before `Layout` into events (input, hover, scrolls, timers,
  effects, styles, pictures), `Layout` into layout, `DisplayList` into display_list; the encoding
  stays the bench's own, after `OffscreenTarget::draw`. The bench counted the change, the flush and
  the styles into events before; on the reference scene events was p95 0.10 ms against 1.0 ms.
- **Allocations per stage.** Today a frame has one count (`crates/bench/src/frame_pipeline.rs:58,70`).
  The clock reads `CountingAllocator::allocations_so_far()` at every boundary, so `FrameSample`
  holds the allocations of each budget stage; the gate still holds the frame total against
  `[frame] allocations`, and its line names the stages of the frame that allocated most, e.g.
  `most 3 per frame (events 3, layout 0, display_list 0, encoding 0)`.
- **The bench and the scheduler.** The bench gets no `FrameScheduler`. The tree asks for a delayed
  frame again in every frame while a caret blinks or a timer waits, by design
  (`crates/core/src/frame_requests.rs:47-59`, `crates/core/src/element_tree.rs:678-680`,
  `crates/core/src/element_tree/text_editing.rs:329-331`); the call into the port costs nothing in
  the tree, and the cost — a new `dispatch_after` per call — lies in the frame loop of a window
  (`crates/platform/src/frame_loop.rs:41-57`). A counting scheduler on an offscreen tree would
  count calls that are right by design and never see a dispatch. Deduplicating the requests in the
  tree is unsafe: the tree cannot tell whether the dispatch it asked for has fired, and a frame
  whose time lies a little before the deadline would wait for a request that is gone.
- **Pictures.** `Renderer::image_status` (`crates/platform/src/render/renderer.rs:159-168`)
  answers the port. The stand still waits for the decoder before its first frame
  (`crates/teststand/src/morok_side.rs:70-81`), so a case is drawn with its pictures in place; the
  run then asks the renderer in every frame, as the host does. A test stage framed without a
  renderer passes a port that panics when asked: such a scene shows no picture awaiting its size.
  `crates/platform/tests/image_pipeline.rs:164-168` renders a picture awaiting its size through
  `Scene::render`, which now asks the renderer.
- **Overlays.** `HostedSurface::overlays` lays a new overlay out before its window exists, so the
  window knows its size (`crates/morok/src/hosted_surface.rs:306-321`); through `lay_out` it sizes
  the pictures too. Placement is no stage: the window of the platform places, and the stage of the
  components tests emulates it. Honest limit: the first size a window is told is not observable
  from the windows of a test — the overlay lays itself out again in its own first frame and asks
  the surface under it to place it again (`hosted_surface.rs:116-120`) — so `frame_run` proves that
  `lay_out` sizes a picture, and the host's use of `lay_out` is checked by reading.
- **What the stand-ins gain.** The stand gets `refresh_hover` and `run_due_timers`: a case whose
  components start a timer (`held_for` in `crates/components/src/tooltip.rs:55` and
  `crates/components/src/toast_stack.rs:91`) sees it fall due at the times `Moment::InMotion`
  frames at. The stage of the components tests gets `refresh_hover` and `size_images`; the bench
  gets hover, timers and pictures.
- **The tests of core.** `Scene::frame_at` of `crates/core/tests/support/mod.rs` and the
  single-stage calls of `image_sizing.rs` and `frame_requests.rs` move onto the run in the first
  phase; the support's port takes the status a test sets and counts what it is asked. The two
  `should_panic` tests of the order — `laying_out_a_tree_whose_styles_are_not_resolved_panics`
  (`crates/core/tests/layout.rs:576`) and
  `building_the_display_list_of_a_tree_that_is_not_laid_out_panics`
  (`crates/core/tests/damage_display_list.rs:738`) — go in the wire phase: no caller outside core
  can call a stage out of order then, and the order is the run's, pinned by `frame_run`. The
  assertions inside the stages stay.
- **The rules of the bench gates and their cases** (`crates/bench/tests/budget_gate.rs` and the
  unit test `stage_clock`):
  - a stage whose 95th percentile exceeds its budget fails and names the scenario and the stage →
    `slowed`: its events are over budget;
  - a frame at or under every budget exits 0 → `fast`;
  - a steady frame that allocates fails → `allocating`;
  - the allocation check names the budget stages the allocations were made in → `allocating`: its
    allocations line names `events` with the count, the scene allocating in its change;
  - warm-up frames are not measured → `slow-start`;
  - each stage of a run counts into the budget stage of its position → `stage_clock`: every
    `FrameStage` and every `SurfaceStage` maps to events before layout, to layout, to display_list;
  - a frame during idle fails the idle gate → `ticking`, its window content on the run of a tree;
  - an unknown scenario exits 2 → `nope`.
  - Limit: the bucket of a run stage is proven by the mapping, not by slowing that stage — a scene
    cannot slow a stage of the kit.
- **Running the gates.** Calls of cargo go one at a time, about 30 s apart, never with `&` (the
  heavy-process hook). Bench times depend on the load of the machine; `bench all` includes
  `cold-start`, which needs a quiet machine. `cargo xtask check` was run at HEAD `bcd3572` before
  this spec: exit 0, the working tree unchanged.

# Gameplan

<!-- 6 phases. `cargo xtask check` closes every one of them. -->

### Phase 1: tree-frame — morok-core runs the stages of a tree in one place

- [ ] Module `frame` of morok-core (`crates/core/src/frame.rs`, `crates/core/src/frame/`):
  `FrameStage`, `StageObserver<Stage>` with `Unobserved`, `TreeFrame`, `FrameOutcome` (see
  Knowledge), exported from the root of the crate.
- [ ] The port `ImageStatuses` (`crates/core/src/media/image_statuses.rs`):
  `image_status(&self, image: ImageId) -> ImageStatus`, exported from the root of the crate.
- [ ] `ElementTree::lay_out` and `ElementTree::run_frame` (see Knowledge): each stage tells the
  observer of its end.
- [ ] The tests of core frame through the run: `Scene::frame_at` of
  `crates/core/tests/support/mod.rs` calls `run_frame` with a port the test sets the status of and
  that counts what it is asked; `hosted_frame` and `a_removed_image_waits_for_its_size_no_longer`
  of `crates/core/tests/image_sizing.rs` and `a_change_made_once_a_frame_began_asks_for_the_next_frame`
  of `crates/core/tests/frame_requests.rs` go through it.
- [ ] `crates/core/tests/frame_run.rs`: `run_frame` tells the ends of `Styles`, `Images`, `Layout`,
  `DisplayList` in that order; `lay_out` tells `Styles`, `Images`, `Layout` and leaves the display
  list as the frame before built it; a picture the port reports decoded takes its natural size in
  the same frame, and the port is asked only for the pictures that await their size; the outcome
  carries the counts of the stages, and a frame that changes nothing counts none; a frame earlier
  than the one before panics.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok-core --test frame_run
```
- The run of a tree keeps the canonical order, sizes pictures in its frame and counts its stages (exit 0).
```timeout=900s
cargo test -p morok-core --test image_sizing
```
- Pictures are sized through the run as the host sizes them (exit 0).
```timeout=900s
cargo test -p morok-core --test frame_requests
```
- A change once a frame began still asks for the next frame (exit 0).
```timeout=1800s
cargo test -p morok-core
```
- Every test of core passes with its scenes framed through the run (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and comment rules are clean (exit 0).

### Phase 2: surface-frame — the surfaces run the frame of a surface in one place (consumes phase 1: the run of a tree)

- [ ] `SurfaceStage` and `SurfaceFrame` (`crates/components/src/view/surface_stage.rs`,
  `crates/components/src/view/surface_frame.rs`), exported by `view`.
- [ ] `Surfaces::run_frame` (see Knowledge): the stages of the surface, then `ElementTree::run_frame`
  with the observer adapted to `SurfaceStage::Tree`.
- [ ] The stage of the components tests: `frame_of` (`crates/components/tests/support/mod.rs:271`)
  calls `Surfaces::run_frame`; the stage holds a port whose status `host_reports` of
  `crates/components/tests/image.rs` sets instead of calling `size_images`;
  `Stage::frame_observed(observer)` frames the surface once with an observer.
- [ ] `crates/components/tests/surface_frame.rs`: the run tells `Hover`, `Scrolls`, `Timers`,
  `Effects` and the four stages of the tree in that order; a `held_for` whose delay runs out shows
  what its signal binds in the same frame; a pointer resting over a box the last frame moved under
  it hovers it in the next frame; an effect that closes the surface in the flush makes the run
  answer `Closed` with no stage of the tree told; an effect that keeps waking itself makes it
  answer the `FlushCycle`.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok-components --test surface_frame
```
- The run of a surface keeps the canonical order on the real run of the tree, timers and hover among its stages (exit 0).
```timeout=900s
cargo test -p morok-components --test image
```
- A picture the host reports is sized by the run of the stage (exit 0).
```timeout=900s
cargo test -p morok-components --test rebind_allocations
```
- A frame through the run still allocates nothing on the hot path (exit 0).
```timeout=1800s
cargo test -p morok-components
```
- Every test of the components passes with the stage on the run (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and comment rules are clean (exit 0).

### Phase 3: host-frame — the windows of an application run the frame through the surfaces (consumes phase 2: the run of a surface)

- [ ] `Renderer` implements `ImageStatuses` through the `image_status` it has
  (`crates/platform/src/render/renderer.rs:159`).
- [ ] `HostedSurface::frame` calls `Surfaces::run_frame` with the shaper and the renderer of the
  application and `Unobserved`; a `FlushCycle` panics with its message, `Closed` returns before
  `show`; `ApplicationState::flush` (`crates/morok/src/application_state.rs:36`) goes.
- [ ] `HostedSurface::overlays` lays a new overlay out through `ElementTree::lay_out` with the
  renderer as the port.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok --test images
```
- The windows of an application size their pictures through the run (exit 0).
```timeout=900s
cargo test -p morok --test overlay_windows
```
- Overlays open, are placed and close with their preliminary layout on the run (exit 0).
```timeout=900s
cargo test -p morok --test button_interaction
```
- Input reaches the components and shows in the frame the run draws (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and comment rules are clean (exit 0).

### Phase 4: stand-frame — the stand and the test stages of the platform frame through the runs (consumes phase 3: the renderer as the port of picture statuses)

- [ ] `Scene::frame` of the stand (`crates/teststand/src/scene.rs:269`) takes the port and calls
  `Surfaces::run_frame`; a `FlushCycle` panics as `Scene::flush` does; `MorokSide::render` passes
  its renderer, and the `size_images` before the frames (`crates/teststand/src/morok_side.rs:53-54`)
  goes.
- [ ] `Scene::frame` of `crates/platform/tests/support/mod.rs` calls `ElementTree::run_frame` with
  a port of the support that panics when asked, and `Scene::render` calls it with the renderer;
  `FieldWindow::frame` of `crates/platform/tests/text_input_client.rs` calls `Surfaces::run_frame`.

**Done when (EXECUTABLE):**
```timeout=1800s
cargo run -p morok-teststand -- check all
```
- Every suite matches Chrome within the thresholds with the MOROK side on the run of the application (exit 0).
```timeout=900s
cargo test -p morok-platform --test image_pipeline
```
- Pictures drawn offscreen are sized through the renderer as the port (exit 0).
```timeout=900s
cargo test -p morok-platform --test text_input_client
```
- The input client edits a field whose window frames through the run (exit 0).
```timeout=900s
cargo test -p morok-platform --test offscreen_boxes
```
- Boxes drawn offscreen still land where the run lays them out (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and comment rules are clean (exit 0).

### Phase 5: bench-frame — the bench measures the frame of the application stage by stage (consumes phases 1-3: the runs and the renderer as the port)

- [ ] `crates/bench/src/stage_clock.rs`: `StageClock` implements `StageObserver<FrameStage>` and
  `StageObserver<SurfaceStage>` and buckets the stages by position (see Knowledge); its unit tests
  map every `FrameStage` and every `SurfaceStage` to its budget stage.
- [ ] `FrameSample` holds the time and the allocations of each budget stage, `allocations()` their
  sum; the allocation check of `crates/bench/src/frame_gate.rs` names the allocations per stage of
  the frame that allocated most.
- [ ] `SurfacePipeline` keeps a clone of the renderer and frames through `Surfaces::run_frame` with
  its clock; `FramePipeline` frames through `ElementTree::run_frame` with its clock;
  `SceneContent` frames through `ElementTree::run_frame` with `Unobserved` and the renderer
  `crates/bench/src/idle_run.rs` gives it from the application.
- [ ] `crates/bench/tests/budget_gate.rs`: the `allocating` case expects its allocations line to
  name `events`.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok-bench --lib stage_clock
```
- Every stage of both runs counts into the budget stage of its position (exit 0).
```timeout=1800s
cargo test -p morok-bench --test budget_gate
```
- The gates fail over budget, name the stage that allocated, skip the warm-up and pass a frame within budget (exit 0).
```timeout=1800s
cargo run --release -p morok-bench -- frame
```
- The reference scene and the screens of the catalog meet the budgets on the frame of the application (exit 0).
```timeout=1800s
cargo run --release -p morok-bench -- scroll-1m
```
- The million rows scroll within budget through the run of a surface (exit 0).
```timeout=1800s
cargo run --release -p morok-bench -- scroll-list
```
- The list of documents scrolls within budget through the run of a surface (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and comment rules are clean (exit 0).

### Phase 6: wire — the stages have one owner on the main path (consumes phases 1-5)

- [ ] `resolve_styles`, `size_images`, `compute_layout` and `build_display_list` of `ElementTree`
  become `pub(crate)`, and so do `refresh_hover`, `follow_scrolls` and `run_due_timers` of
  `Surface`; the two `should_panic` tests of the order go (see Knowledge).
- [ ] `docs/MOROK-ARCHITECTURE.md` (§2, "Frame stages") names the two runs, `ElementTree::run_frame`
  and `Surfaces::run_frame`, as the one owner of the order the host, the bench and the stand call.
- [ ] `crates/catalog/tests/wire_frame_run.rs` (`harness = false`, a `[[test]]` in
  `crates/catalog/Cargo.toml`) on the real modules: on the Overlays page the pointer rests over the
  button of the tooltip «Recalculate the sizes of every folder», and after its resting time the
  window of the tooltip opens with that text; the pointer leaves and the window closes. Events
  reach the window as the system posts them; nothing is mocked above the system boundary.

**Done when (EXECUTABLE):**
```timeout=1800s
cargo test -p morok-catalog --test wire_frame_run
```
- Hover, a timer and the layout of an overlay run through the frame of the application in the catalog (exit 0).
```timeout=1800s
cargo test -p morok-catalog --test wire_main_path
```
- The main path of the debts still holds on the run (exit 0).
```timeout=1800s
cargo run -p morok-teststand -- check all
```
- Every suite matches Chrome within the thresholds (exit 0).
```timeout=1800s
cargo run --release -p morok-bench -- all
```
- Every budget holds on the frame of the application (exit 0).
```
grep -q Surfaces::run_frame docs/MOROK-ARCHITECTURE.md
```
- The architecture names the owner of the order (exit 0).
```timeout=900s
cargo xtask check
```
- No stage is called outside its run anywhere in the workspace, and formatting, clippy and comment rules are clean (exit 0).
