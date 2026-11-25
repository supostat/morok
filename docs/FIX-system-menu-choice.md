<!--
  FIX-system-menu-choice.md — the menu step of crates/morok/tests/overlay_windows.rs chooses its
  command through the menu the system tracks, so a moving pointer no longer decides it. 1 phase.

  DECISION BOUNDARY (hypothesis fan of 2026-11-25, hypothesis 1 chosen on the recommendation): a
  real pointer that moves between the queued down arrow and Return takes the highlight off the
  first command, and Return closes the menu with nothing chosen. Rejected: hypothesis 2 — the menu
  opens later than the 1.4 s the step waits: with every one of 12 cores busy the test passed, and
  the menu window showed about 1.4 s before the test ended.
-->

# Baseline

- HEAD is `d39d820` on `main`. Run 3b276f1c of `docs/SPEC-morok-frame.md` escalated at
  `host-frame`: its first gate failed only on this check; its second failed to compile on a target
  directory a worktree had shared, since repaired.
- The working tree holds the uncommitted work of `host-frame` in
  `crates/morok/src/application_state.rs`, `crates/morok/src/hosted_surface.rs` and
  `crates/platform/src/render/renderer.rs`; this fix touches none of them.
- The failure is older than that work: at `d39d820`, three window tests run in a row failed with
  the same message.
- Version: no release automation, nothing is published, the workspace stays at `0.1.0`, with no
  version phase.

# Stack

- No new crates. The `objc2-foundation` dev-dependency of `morok` gains the feature
  `NSNotification` (`crates/morok/Cargo.toml`).

# Conventions

- Production code is not touched: the defect lies in how the test drives a menu of the system.
- A window test of `morok` is a `harness = false` target that reports through `Expectations` at
  the end of one run of the application.
- Comments are English, rare, and never mention phases, tasks or this spec; `docs/` is English.

# Knowledge

- **The symptom.** `cargo test -p morok --test overlay_windows` fails now and then with:
    overlay windows failed:
    the command chosen in the menu of the system runs: []
- **The reproduction.** A pointer moved by a script during the test: osascript -l JavaScript
  posts `CGEventCreateMouseEvent(null, kCGEventMouseMoved, point, 0)` through
  `CGEventPost(kCGHIDEventTap, …)` every 5 ms, two points left and right of `NSEvent.mouseLocation`.
  Moves every 5 ms: failed 2 of 2; every 30 ms: failed 1 of 2; no moves: passed 3 of 3, once with
  every core busy.
- **Ruled out.** The windows of the test lie on another Space behind a full-screen editor in
  passing runs too: an observer of `CGWindowListCopyWindowInfo` saw them and the window of the menu
  (layer 101) off screen while the test passed. A load on every core does not fail it.
- **The mechanism.** The step queues the down arrow and Return in the application
  (`crates/morok/tests/overlay_windows.rs:442-445`, `queue_key_press` in
  `crates/morok/tests/support/mod.rs:237-244`) and the menu pops up with no item under the pointer
  (`popUpMenuPositioningItem_atLocation_inView(None, …)`, `crates/platform/src/native_menu.rs:32`).
  `NSMenu` tracks the real pointer as well: a move between the two keys takes the highlight off the
  first command, and Return closes the menu with nothing chosen.
- **The way to choose.** An observer object of the test (`define_class!`, as `StagedDrag` in
  `crates/morok/tests/drag_windows.rs:42`) registered for `NSMenuDidBeginTrackingNotification`
  with `addObserver_selector_name_object` (as `crates/platform/src/surface_view.rs:533-545`). The
  object of the notification is the menu: the observer records the titles of its items, performs
  the action of its first item with `performActionForItemAtIndex` and ends the tracking with
  `cancelTracking`; `pop_up` then hands the chosen id to the surface as a choice by the user does.
  Not verified: that both calls work inside the handler of the notification.
- **What the gate cannot prove.** A gate command is one process and cannot move the pointer while
  the test runs; the phase proves the step under the script of the reproduction instead.
- **Not reproduced.** The check «the arrow of the pushed popover points at its opener where it
  would if nothing pushed it: the arrow is at None, expected at 1972» failed once, at 06:25, and
  held in the eight runs after; it stays out of this fix.

# Gameplan

<!-- 1 phase. The regression test of the menu closes it. -->

### Phase 1: system-menu-choice — the menu step chooses its command through the tracked menu

- [ ] `crates/morok/tests/overlay_windows.rs`: the step of the menu of the system registers an
  observer of `NSMenuDidBeginTrackingNotification` that records the titles of the menu, performs
  the action of its first item and cancels the tracking; it checks that the menu shows `Reveal`
  and `Rename` and that the chosen command is `reveal`; no key is queued for the menu, and the
  watchdog names a menu that never closed.
- [ ] `crates/morok/tests/support/mod.rs`: `queue_key_press` goes; the menu step was its one user.
- [ ] `crates/morok/Cargo.toml`: the `objc2-foundation` dev-dependency gains `NSNotification`.
- [ ] Under the pointer script of the reproduction the test passes three runs of three; the runs
  are recorded with the harvest.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok --test overlay_windows
```
- The menu of the system shows the commands of the component and the chosen one runs, beside the other overlays of the test (exit 0).
```timeout=60s
grep -q NSMenuDidBeginTrackingNotification crates/morok/tests/overlay_windows.rs
```
- The menu step chooses through the menu the system tracks (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and comment rules are clean (exit 0).
