<!--
  SPEC-morok-delayed-frames.md — a window queues one block on the main queue per moment a tree asks
  a frame for, not one per frame that asks again. 1 phase.

  DECISION BOUNDARY (fan of 2026-11-25, option B chosen on the recommendation): the port of the
  scheduler carries the moment of the frame, on the clock of the frames of the tree, and the frame
  loop of a window keeps the moment its block waits for. Rejected: A — the frame loop works the
  moment out from the delay and the time of its own frame: the port stays, but the base of the
  delay becomes an unwritten contract, and a request made outside the window's own frame would get
  another base. C — the tree stops asking while its moment has not come: the tree cannot tell
  whether the block of the window has fired, so a frame a little before the moment would wait for
  a request that is gone (docs/SPEC-morok-frame.md, Knowledge "The bench and the scheduler").
-->

# Baseline

- Prior spec: `docs/SPEC-morok-frame.md` — all 6 phases closed (run 0ddbbf67), committed up to
  `7c03e3d`, pushed to `origin/main`.
- Prior spec-hash: sha256:<placeholder — confirm>
- The debt: `docs/SPEC-morok-frame.md` left out of scope that the frame loop of a window queues a
  new block on the main queue for every delayed frame request
  (`crates/platform/src/frame_loop.rs:41-57`), while the tree asks again in every frame as long as
  a caret blinks or a timer waits (`crates/core/src/frame_requests.rs:47-59`). The behaviour is
  right — a counter makes every block but the last ask for nothing — the work is not: while frames
  run for another reason, every frame queues a block.
- Exists and is not touched: what the stages of a frame do and their order
  (`ElementTree::run_frame`, `Surfaces::run_frame`); `FrameScheduler::request_frame` and the
  requests a change makes; the display link starting on a request and pausing after a frame with
  no new request (`FrameLoop::request_frame`, `begin_frame`, `end_frame`); the half period of the
  caret and when a timer falls due; the bench, which gives its trees no scheduler; the budgets in
  `crates/bench/budgets.toml`.
- Version: no release automation (no `.github/`, `.cargo/config.toml` holds only the `xtask`
  alias) and nothing is published: the workspace stays at `0.1.0`, with no version phase, although
  the public port `FrameScheduler` changes a method and `FrameStatistics` gains a field.
- Rationale of the choice — staged decision note (id after accept).

# Stack

- No new crates and no new dependencies.
- `morok-core`: `FrameScheduler::request_frame_at(time)` replaces `request_frame_after(delay)`;
  `FrameRequests` passes on the earliest moment asked for in a frame; the caret and the timers
  give the moment of their next change.
- `morok-platform`: `FrameLoop` keeps the moment its block waits for; `FrameStatistics` counts the
  blocks the frame loop queued; the feature of `objc2-quartz-core` that exposes
  `CACurrentMediaTime`, when it is not enabled yet; the test target `delayed_frames`
  (`harness = false`) in `crates/platform/Cargo.toml`.

# Conventions

- Every convention of `docs/SPEC-morok.md`, `docs/SPEC-morok-debts.md` and
  `docs/SPEC-morok-frame.md` holds: no heap allocation on the hot path of a frame; window tests run
  with the display kept on; the stand thresholds and the bench budgets do not change.
- A behaviour that changes updates, in the same phase, the test that pinned the old behaviour.
- Comments are English, rare, and never mention phases, tasks or this spec; `docs/` is English.

# Knowledge

- **One clock.** The time of a tree is the `now` its host passes into a frame. A window passes the
  `targetTimestamp` of its display link (`crates/platform/src/surface_view.rs:150`), which is the
  host time of Core Animation, the clock `CACurrentMediaTime` reads. A moment the tree asks a frame
  for is on that clock, so the frame loop turns it into a wait as the moment minus
  `CACurrentMediaTime()`; a moment already past asks for a frame at once and queues nothing.
- **Exact moments.** The caret turns at the moment it was shown plus a whole number of half
  periods (`crates/core/src/editing/caret_blink.rs:21-29`); a timer falls due at `due_at`, fixed
  by the first frame after it was started (`crates/core/src/timing/timers.rs:53`). Both are worked
  out in `Duration` arithmetic, so every frame before the moment asks for the very same moment,
  and equality is the test of a repeat, with no tolerance.
- **The frame loop.** It keeps the moment its block waits for. The same moment again queues
  nothing. Another moment queues a new block and raises the counter that makes the earlier block
  ask for nothing, as `delayed_requests` does now (`crates/platform/src/frame_loop.rs:42-53`). The
  block that fires forgets the moment, so the next request queues again. The window knows whether
  its block fired; the tree does not, which is why the repeat is dropped in the window.
- **What a frame asks.** `FrameRequests` passes on the earliest moment asked for since the frame
  began (`crates/core/src/frame_requests.rs:51-59` keeps the shortest delay now);
  `ElementTree::resolve_styles` asks for the moment the next timer falls due
  (`crates/core/src/element_tree.rs:727-729`) and `advance_caret` for the moment of the next turn
  (`crates/core/src/element_tree/text_editing.rs:329-331`). The tree still asks in every frame:
  the call costs nothing in the tree.
- **Test schedulers.** Five test targets of core implement the port:
  `crates/core/tests/frame_requests.rs:27`, `crates/core/tests/damage_display_list.rs:774`,
  `crates/core/tests/image_sizing.rs:31`, `crates/core/tests/animation_timeline.rs:36` and
  `crates/core/tests/timers.rs:26`; the last records what the tree asks, and its expectations
  compare moments once the port carries them.
- **Proving the repeat is gone.** `FrameStatistics` (`crates/platform/src/frame_statistics.rs`)
  counts display-link callbacks and presented frames; a third count, the blocks the frame loop
  queued, makes a repeat visible. `crates/platform/tests/display_link_idle.rs` is the model of a
  window test (`harness = false`, `support::run_main_loop`, `Expectations`). A text field mounted
  and focused as in `crates/platform/tests/text_input_client.rs:53-79` blinks; an element with an
  endless CSS animation (`1s linear infinite`, as `crates/core/tests/animation_timeline.rs:507`
  uses) keeps the display link running. While both run, the callbacks greatly outnumber the turns
  of the caret, and the blocks number the turns plus one at most. Mutation for the gate: the frame
  loop queues a block whatever the moment — the blocks then follow the callbacks and the test
  turns red.
- **Display.** `CADisplayLink` does not tick while the display is off; window tests and gates run
  under `caffeinate -d -u`.
- **Lint at HEAD.** `cargo xtask check` exits 0 at `7c03e3d`; the working tree is unchanged
  before and after the run.

# Gameplan

<!-- 1 phase. The port changes in core and platform at once, so the work is one phase. -->

### Phase 1: frame-moments — A window queues one block per moment a tree asks a frame for

- [ ] `FrameScheduler` (`crates/core/src/frame_scheduler.rs`): `request_frame_at(&self, time)`
  replaces `request_frame_after`: a frame at `time`, on the clock of the `now` the host passes to
  the frames of the tree, in place of the frame asked for this way before that has not run.
- [ ] `FrameRequests` passes on the earliest moment asked for since the frame began; the caret
  gives the moment of its next turn and the timers the moment the next one falls due
  (`caret_blink.rs`, `timers.rs`, `element_tree.rs`, `element_tree/text_editing.rs`), their unit
  tests following.
- [ ] The five test schedulers of core take the new method; `crates/core/tests/timers.rs` compares
  moments and gains a test that the frames before a turn of the caret ask for the moment of that
  turn, the same in each.
- [ ] `FrameLoop` (`crates/platform/src/frame_loop.rs`) keeps the moment its block waits for: the
  same moment queues nothing; another queues a new block whose wait is the moment minus
  `CACurrentMediaTime()`; a moment already past asks for a frame at once; the block that fires
  forgets the moment. `FrameRequester` passes the moment through.
- [ ] `FrameStatistics` gains the count of the blocks the frame loop queued, filled by
  `SurfaceView::statistics`.
- [ ] `crates/platform/tests/delayed_frames.rs` (`harness = false`, `[[test]]` in
  `crates/platform/Cargo.toml`): a focused text field beside an element with an endless animation
  — over a stretch the callbacks are many times the turns of the caret and the queued blocks
  number the turns plus one at most; without the animation the caret alone presents a frame at
  every turn and takes no more than two callbacks per turn; once the field loses focus the window
  takes at most one callback and queues no block.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok-platform --test delayed_frames
```
- A window queues one block per turn of its caret while an animation runs its frames, the caret alone still blinks, and a window without focus falls idle (exit 0).
```timeout=900s
cargo test -p morok-platform --test display_link_idle
```
- A window with no damage still takes no display-link callback, and damage still starts the link for one frame (exit 0).
```timeout=900s
cargo test -p morok-platform --test text_input_client
```
- A focused field of a window still takes text input and shows its caret (exit 0).
```timeout=900s
cargo test -p morok-core --test timers
```
- The tree asks for the moment its caret turns or its timer falls due, the same moment in every frame before it (exit 0).
```timeout=900s
cargo test -p morok-core
```
- Every test of core, the five test schedulers among them, passes (exit 0).
```timeout=900s
cargo test -p morok-catalog --test wire_frame_run
```
- In the catalog a tooltip still opens once the pointer has rested, its timer driven through the window (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and comment rules are clean (exit 0).
