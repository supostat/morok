<!--
  SPEC-morok-rebind.md — OptionList and Tree rebind their row slots without heap allocations, as
  List does since docs/SPEC-morok-debts.md. 1 phase.

  DECISION BOUNDARY (fan of 2026-11-24, option A chosen on the recommendation): the way of List —
  one buffer per row slot, written and set by an effect — proven by a components test with a
  counting allocator. Rejected: B — the same code proven by bench scenarios `scroll-tree` and
  `scroll-options`: the bench pipeline frames one surface and would need overlay framing and scenes
  of its own, out of proportion with the debt. C — labels as `Rc<str>`: `Mount::text` and
  `accessible_name` take `Property<String>`, so a new property kind and a second way to rebind a
  row beside the way of List.
-->

# Baseline

- Prior spec: `docs/SPEC-morok-debts.md` — all 11 phases closed (run 4349f514), committed up to
  `8aa8010`; HEAD is `3b2e8a0`, pushed to `origin/main`.
- Prior spec-hash: sha256:adb1e542ee0f593a894e85497c93f8c36c2fa88497158f5a6de564525ba63343
- The debt: `docs/SPEC-morok-debts.md` left out of scope that `OptionList`
  (`crates/components/src/option_list.rs:62-66`) and `Tree` (`crates/components/src/tree.rs:84-90`)
  clone a label on every rebind; `Tree` also clones its `ShownItem` and the key of the row.
- Exists and is not touched: `List` and its `label_of` (`crates/components/src/list.rs:110-117`);
  `VirtualRows`; the public API of `Tree` (`TreeItem { key, label: String, children }`,
  `Tree::new`, `label`), of `Select` (`option(value, name)`) and of `Combobox` (`option(name)`);
  the voice of the combobox (`spoken_highlight`, `crates/components/src/combobox.rs:200`), which
  formats on a change of the highlight, not on a rebind; the event handlers of `Tree` that clone a
  key or an item when a click or a key arrives.
- Version: no release automation (no `.github/`, `.cargo/config.toml` holds only the `xtask`
  alias), nothing is published, and no public signature changes: the workspace stays at `0.1.0`,
  with no version phase.
- Rationale of the choice — staged decision note (id after accept).

# Stack

- No new crates and no new dependencies.
- `morok-components`: `OptionList` and `Tree` write their row labels into a buffer of the row
  slot; the test target `rebind_allocations` (`harness = false`) in `crates/components/Cargo.toml`.

# Conventions

- Every convention of `docs/SPEC-morok.md` and `docs/SPEC-morok-debts.md` holds: no heap
  allocation on the hot path of a frame; etalons come only from `etalons render`; the stand
  thresholds do not change.
- One way to rebind a row slot across the kit: the way of `List`.
- A behaviour that changes updates, in the same phase, the test that pinned the old behaviour.
- Comments are English, rare, and never mention phases, tasks or this spec; `docs/` is English.

# Knowledge

- **The way of List.** A row slot of `List` keeps one `String`; its effect clears it, lets
  `label_of` write the label of the row the slot shows, and sets the text of the label and the
  accessible name of the row from it (`crates/components/src/list.rs:110-117`).
  `ElementTree::set_text` returns at once on equal text and otherwise clears and pushes into the
  element's own string (`crates/core/src/element_tree.rs:413-425`), so a rebind allocates only
  when a label outgrows the capacity it has had.
- **OptionList.** `mount_row` reads the option of its row from `shown` into a memo of
  `Option<usize>`, which allocates nothing; the memo `name` clones `labels[option]`
  (`crates/components/src/option_list.rs:62-66`) and feeds both the text of `.option-name` and
  the accessible name of the row. `Dropdown.labels` is an `Rc<[String]>` the slot reads from
  (`crates/components/src/dropdown.rs:29`). A slot whose row lies past a list that shrank finds no
  option and shows no label.
- **Tree.** Each slot builds a memo `item` that clones the whole `ShownItem` with its key and
  label (`crates/components/src/tree.rs:84-87`), a memo `text` that clones the label again
  (`:88`), and `is_selected` clones the key to compare it with the selected one (`:89-92`). The
  level, the open state and whether the item has children are `Copy` and are read from `shown` by
  the slot's index. `shown` is rebuilt only when `expanded` changes (`shown_items`), never on a
  scroll. The row of the selected key among the shown items, as one memo every slot reads, turns
  the per-slot key comparison into a comparison of indices.
- **Proving no allocation.** `crates/reactive/tests/steady_allocations.rs` counts allocations
  with a global allocator in a `harness = false` target and is the model. A `harness = false`
  target is a `[[test]]` entry in `crates/components/Cargo.toml`; its `main` uses `mod support;`
  as the other targets do. `Stage` of the components tests frames overlays, but
  `Stage::overlays_of` returns a new `Vec` each call, so a measured frame goes through a path of
  the stage that allocates nothing of its own. `OneLineShaper` clears and extends the buffers it is
  given: a label longer than any the warm-up showed grows a buffer once, so the labels of the test
  are of one length (`Option 0000` to `Option 0999`, `Folder 0000` to `Folder 0999`). A frame that
  rebinds nothing allocates nothing either way, so every measured frame shows a first row in view
  that the frame before did not, and the test checks it.
- **Names on rebind.** `crates/components/tests/virtualization.rs` pins for `List` that a scroll of
  one row binds one slot to the text and the accessible name of the row that came into view. The
  same check is new for the tree (`crates/components/tests/collections.rs`) and the option list
  (`crates/components/tests/select_combobox.rs`, beside
  `a_list_of_a_thousand_options_holds_the_rows_in_view_and_scrolls_to_the_highlight`).
- **Etalons.** The stand renders the markup of a case from its tree, after the effects ran; a label
  set by an effect instead of a memo gives the same markup. Should `check` report a case `STALE`
  because its markup changed, the suite is re-rendered with
  `cargo run -p morok-teststand -- etalons render <suite>` and the diff is read before the commit:
  a PNG that changes is a defect, not a re-render.
- **Lint at HEAD.** `cargo xtask check` exits 0 at `3b2e8a0`; the working tree is unchanged before
  and after the run.

# Gameplan

<!-- 1 phase. `cargo xtask check` closes it. -->

### Phase 1: rebind-rows — OptionList and Tree rebind their row slots without allocations

- [ ] `OptionList` (`crates/components/src/option_list.rs`): each row slot keeps one `String`; an
  effect clears it, writes the label of the option the row shows (none past the shown options)
  and sets the text of `.option-name` and the accessible name of the row from it; the memo `name`
  goes.
- [ ] `Tree` (`crates/components/src/tree.rs`): a row slot reads the level, the open state and
  whether the item has children from `shown` by its index; its label is written into a buffer of
  the slot and set as the text of the row and its accessible name; whether a row is selected
  compares the slot's index with a memo of the row of the selected key among the shown items. No
  memo clones a `ShownItem`, a label or a key on a rebind.
- [ ] `crates/components/tests/rebind_allocations.rs` (`harness = false`, `[[test]]` in
  `crates/components/Cargo.toml`) with a counting global allocator: a `Tree<String>` of 1000
  items and an open `Select` of 1000 options, labels of one length; the wheel scrolls each through
  its rows; every measured frame shows a first row in view the frame before did not, and the
  measured frames of the surface of the tree and of the overlay of the list make no heap
  allocation.
- [ ] `crates/components/tests/collections.rs`: a scroll of the tree by one row binds one slot to
  the text and the accessible name of the item that came into view, and the selected item stays
  marked when a scroll moves it to another slot.
- [ ] `crates/components/tests/select_combobox.rs`: a scroll of an open list by one row binds one
  slot to the text and the accessible name of the option that came into view, and the chosen
  option keeps its check mark when a scroll moves it to another slot.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok-components --test rebind_allocations
```
- Scrolling a tree of a thousand items and an open list of a thousand options rebinds a slot every frame with no heap allocation (exit 0).
```timeout=900s
cargo test -p morok-components --test collections
```
- The tree binds its slots to the text, the name and the selection of the items in view (exit 0).
```timeout=900s
cargo test -p morok-components --test select_combobox
```
- The option list binds its slots to the text, the name and the check mark of the options in view (exit 0).
```timeout=900s
cargo run -p morok-teststand -- check collections
```
- Lists, trees and tabs still match Chrome in both themes (exit 0).
```timeout=900s
cargo run -p morok-teststand -- check select-combobox
```
- Selects and comboboxes, open lists among them, still match Chrome in both themes (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and comment rules are clean (exit 0).
