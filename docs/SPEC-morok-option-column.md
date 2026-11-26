<!--
  SPEC-morok-option-column.md — the list a combo box drops down keeps no empty column at the left
  of its rows: whether a list has the column of the check mark is a property of the list, said by
  the control that owns it. 1 phase.

  DECISION BOUNDARY (fan of 2026-11-26, option B chosen on the recommendation): the column is
  static — the list of a select marks the option it holds and keeps the column, the list of a
  combo box marks nothing and has none. Rejected: A — the column comes and goes with the mark: the
  names of a combo box would move by 22 px whenever the typed text comes to name an option in full
  or stops naming one. C — the mark at the right edge of both lists: it changes the look of the
  select, which nothing asked for. Reference: in Radix Primitives an item indicator is rendered
  only for the selected item, and the room for it is the stylesheet's (its documentation, read on
  2026-11-26; how its demo stylesheets lay the mark out was recalled, not read again).
-->

# Baseline

- Prior spec: `docs/SPEC-morok-guide.md` — its four phases closed (run e819dc4a), committed as
  `bc72b5f`, `385769a`, `9c1a6a2` and `0e0f579`; HEAD is `0e0f579`, pushed to `origin/main`.
- Prior spec-hash: sha256:9f1084423e93268c31f51394b310ce6e40b702a99a2509b9440b6f415b44a750
- Exists and is not touched: the look and the behaviour of `Select`; the keys, the highlight and
  the filter of both lists; how a combo box opens and chooses; every public signature; the
  accessibility tree — the row of the option a control holds still tells it is selected.
- Changes: the open list of a `Combobox` shows no check mark, and the names of its rows stand at
  the inset of the row.
- Out of scope: icons of options; a mark at the right edge; `Menu` and its items.
- Version: no release automation (no `.github/`) and nothing is published: the workspace stays at
  `0.1.0`, with no version phase.
- Rationale of the choice — staged decision note (id after accept).

# Stack

- No new crates.
- `morok-components`: `crates/components/src/chosen_mark.rs` (new) — `ChosenMark`, whether a list
  marks the row of the option its control holds: `CheckMark` or `NoMark`; `dropdown.rs` — a field
  of `Dropdown`, given to `Dropdown::new`; `select.rs` and `combobox.rs` — each names its own;
  `option_list.rs` — the row of a list that marks holds the mark and carries a class;
  `option_list.css` — the class `with-mark`.
- `crates/components/tests/select_combobox.rs` — the cases.
- `crates/teststand/etalons/select-combobox` — rendered again.

# Conventions

- Every convention of the earlier specs holds: etalons come only from `etalons render`; the stand
  thresholds and the bench budgets do not change; a row slot is bound to another option with no
  layout and no allocation.
- The column of the mark is decided once, when the list is built: nothing a user does adds or
  removes it while the list is open.
- No public signature changes.

# Knowledge

- **The list.** `Select` and `Combobox` share `OptionList` (`crates/components/src/option_list.rs`)
  through `Dropdown` (`crates/components/src/dropdown.rs`). A row is a slot of `VirtualRows`:
  `mount_row` builds it once, and the slot is bound to another option as the list scrolls or is
  filtered.
- **Why the name is positioned.** `.option-name` is `position: absolute` between fixed insets
  (`crates/components/src/option_list.css:46-57`), so that a new name lays out no box. The column
  of the mark is therefore the `left` of the name, `calc(var(--space-2) * 2 + 14px)` — 30 px —
  and not a gap of a flex row. Without the column the name stands at `var(--space-2)`, 8 px.
- **Who marks.** The marked row is `Dropdown.chosen`. A select holds the position of its value
  among its options, nearly always some (`crates/components/src/select.rs:126`). A combo box holds
  the option its text names in full, none while the user types
  (`crates/components/src/combobox.rs:156`): at an empty field every row keeps an empty column.
- **Why static.** A column that came with the mark would move every name while the user types.
  So the control that owns the list says once whether the list marks.
- **The class is on the row.** A selector takes one child step, and a row lies four elements
  under the panel: a class on `.options` cannot reach `.option-name`. The rule is
  `.option.with-mark > .option-name`.
- **The state stays.** `States::CHECKED` on the row of the chosen option stays in both lists: the
  role `ListBoxOption` tells it as selected. Only the mark and its column go from the combo box.
- **The helper of the tests.** `name_of` in `crates/components/tests/select_combobox.rs:84` reads
  a row as its mark and its name; the rows of a combo box hold the name alone.
- **The stand.** The suite `select-combobox` has four cases. A change of `option_list.css` makes
  all four stale; `cargo run -p morok-teststand -- etalons render select-combobox` renders them
  again in Chrome. Of the four pictures `combobox-open` alone is expected to change; a changed
  picture of another case is read and explained before the commit.
- **Rebinding.** `crates/components/tests/rebind_allocations.rs` runs an open select of a thousand
  options and counts allocations per frame: what a row is built of must not add work to a frame.
- Option column · the list of a combo box holds no mark → every row of an open combo box has one
  child, its name, and the box of the name starts 8 px from the left edge of the row.
- Option column · a text that names an option in full adds no mark → with such a text the row of
  that option has one child still, and is in the state `checked`.
- Option column · the list of a select keeps its mark and its column → every row of an open select
  holds the mark and the name, and the box of the name starts 30 px from the left edge of the row.
- **Lint at HEAD.** `cargo xtask check` exits 0 at `0e0f579`, the working tree unchanged before
  and after.

# Gameplan

<!-- One phase: the change is one property of one list. -->

### Phase 1: option-column — The list of a combo box has no column of a mark

- [ ] `ChosenMark` in a file of its own; `Dropdown` holds it and `Dropdown::new` takes it; `Select`
  gives `CheckMark`, `Combobox` gives `NoMark`.
- [ ] `mount_row`: in a list that marks, the row holds the check mark and carries the class
  `with-mark`; in another it holds its name alone. The state `checked` is set in both.
- [ ] `option_list.css`: the name at `var(--space-2)`; past the mark only under
  `.option.with-mark > .option-name`. The comment at the head of the file says which list marks.
- [ ] The doc comments that say a row is marked — of `OptionList`, of `Dropdown.chosen` and of
  the list of a combo box — say what holds now.
- [ ] `crates/components/tests/select_combobox.rs`: the three cases of Knowledge, a test for each;
  `name_of` reads the rows of either list.
- [ ] The etalons of `select-combobox` rendered again; the pictures that changed are named.
- [ ] Mutations, one a run, each taken back: `Combobox` given `CheckMark` turns the two cases of
  the combo box red; the rule `.option.with-mark > .option-name` removed turns the case of the
  select red.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok-components --test select_combobox
```
- The list of a combo box holds no mark and no column, and the list of a select keeps both (exit 0).
```timeout=900s
cargo test -p morok-components --test rebind_allocations
```
- A row slot is still bound to another option with no allocation (exit 0).
```timeout=900s
cargo run -p morok-teststand -- check select-combobox
```
- The kit draws the four cases of the suite as their etalons, rendered again, show them (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy, the comment rules and the documentation checks are clean (exit 0).
