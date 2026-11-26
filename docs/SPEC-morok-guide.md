<!--
  SPEC-morok-guide.md — the documentation an agent builds an application on the kit from: a guide
  that travels with the facade crate and compiles its own examples, a reference of the CSS subset
  rendered from the compiler, documentation on every public item, and checks that hold all three
  to the code. 4 phases.

  DECISION BOUNDARY (fan of 2026-11-26, option A chosen on the recommendation): the reference of
  the API is the rustdoc of the sources, found through an index the guide keeps and a test compares
  with the exports; a plugin of Claude Code is a later spec, after the first application moves to
  the kit. Rejected: B — the same and a plugin with one entry skill now: the skill would be written
  before any port shows which procedures repeat, and its check is presence and judgement. C — the
  same and the `missing_docs` lint on the public crates: the lint asks a line for every field and
  variant too (about 213 public fields have none, variants not counted), lines that say what the
  name already says, against the comment rule of the project. Left out of the fan: a markdown
  reference generated from the sources — rustdoc JSON needs a nightly toolchain (not checked by a
  run) and the toolchain is pinned to 1.94.0.
-->

# Baseline

- Prior spec: `docs/SPEC-morok-delayed-frames.md` — its one phase closed (run 55d81932), committed
  as `f903d42`; HEAD is `2795a7d`, pushed to `origin/main`.
- Prior spec-hash: sha256:d0b1016061d6f74543c6e8317b314a10d3cc428c6d5196aab3c2dbbad98003b8
- Exists and is not touched: the behaviour of the kit and every public signature it has. The spec
  adds public items and changes none: the documentation modules `morok::guide` and the function
  of `morok-css` that renders the reference of its subset.
- Exists: rustdoc on most public items; `docs/MOROK-ARCHITECTURE.md`, which tells how the kit is
  built and stays the document of its makers; the catalog (`crates/catalog`), the one application
  that uses every component; `cargo xtask check` — rustfmt, clippy and the comment rules.
- Missing: no doctest, no README, no document that tells how to use the kit; the CSS the compiler
  accepts is listed only in its own tables.
- Out of scope: a plugin of Claude Code (a later spec); documentation of fields and variants as a
  rule; a generated reference of the Rust API; publishing to crates.io.
- Version: no release automation (no `.github/`, `.cargo/config.toml` holds only the `xtask`
  alias) and nothing is published: the workspace stays at `0.1.0`, with no version phase.
- Rationale of the choice — staged decision note (id after accept).

# Stack

- No new crates in the workspace: `xtask` takes `syn` and `toml` from the workspace dependencies.
- `xtask`: the rule of public documentation, the subcommand `public-docs <workspace directory>`,
  `crates/xtask/tests/public_docs.rs`; `check` runs the documentation checks.
- `morok-css`: `subset_reference()`, its committed output `crates/css/docs/css-subset.md`, and
  `crates/css/tests/subset_reference.rs`.
- `morok`: the chapters `crates/morok/docs/*.md`; `crates/morok/src/guide.rs`, which attaches them
  to rustdoc; `crates/morok/tests/guide.rs`; `morok-css` as a dev-dependency.
- `README.md` at the root; doc comments in the seven public crates.

# Conventions

- Every convention of the earlier specs holds: window tests run with the display kept on, the
  stand thresholds and the bench budgets do not change.
- The documentation is English. It tells how to use the kit, never how the kit was made: no
  history, no phases, no spec.
- A doc comment says what the item is for and what its caller must know — a panic, a unit, the
  thread, the clock — and never repeats the signature in words.
- Rust in the guide is a doctest: no fence is tagged `ignore`, and a fence that opens a window is
  `no_run`. What a doctest cannot show — the build script, the tokens and the stylesheets of an
  application — the guide shows by naming files of the catalog.
- The reference of the subset is never edited by hand: it is rendered again with
  `MOROK_UPDATE_GOLDEN=1 cargo test -p morok-css --test subset_reference`.

# Knowledge

- **The reader.** An agent in the repository of an application reaches the kit through the
  dependency: `cargo metadata --format-version 1` names the manifest of `morok`, the root
  `README.md` leads to the guide. Under a path or a git dependency the whole repository is there,
  so the guide may name files of any crate. A crates.io package would hold `crates/morok` alone:
  a limit that waits for publishing.
- **The facade.** `crates/morok/src/lib.rs` exports `Application`, `Window`, `Styles`, `Runtime`,
  the types of the platform and the layers as the modules `components`, `core`, `reactive` and
  `style`. `Application` (`crates/morok/src/application.rs:36-162`): `new(Styles)`,
  `open_window(&WindowOptions, impl FnOnce(&mut Mount<'_>) -> ElementId)`, `update`, `defer`,
  `set_themes`, `set_appearance`, `add_image`, `set_canvas_painter`, `run`.
- **Doctests.** `Styles::of_kit()` (`crates/morok/src/styles.rs:17`) gives an application the
  tables of the kit alone, so a fence can build an application with no build script.
  `Application::new` needs the main thread and Metal: such a fence is `no_run`, compiled and never
  run. The fences of a file attached with `#[doc = include_str!(…)]` are doctests of the crate.
  To rustdoc an untagged fence is Rust, and so is a block indented by four spaces: every other
  fence names its language (`css`, `toml`, `text`) and nothing is indented into a block.
- **What a doctest cannot show.** The catalog is the application to point at: its manifest
  (`crates/catalog/Cargo.toml` — `morok` as the dependency, `morok-components` and `morok-css` as
  build-dependencies), its build script (`crates/catalog/build.rs`, which calls
  `build_script::generate_application`), its tokens (`crates/catalog/tokens/base.css` defines
  every token of the kit; a missing one is the diagnostic `--x is not defined`), its stylesheets
  (`crates/catalog/styles`), the generated tables it includes (`crates/catalog/src/styles.rs`)
  and the classes it uses (`CLASSES.page` in `crates/catalog/src/page_layout.rs`).
- **Examples that exist.** One page of the catalog per family of components
  (`crates/catalog/src/pages/*.rs`) and two screens (`file_manager.rs`, `disk_map.rs`).
- **The chapters.** `overview.md` — what the kit is and needs, its layers, how to find a thing,
  the chapters; `application.md` — from an empty crate to a window, themes, several windows,
  `defer`; `elements.md` — `Mount` and `ElementBuilder`: elements, text, classes, children,
  properties bound to signals, handlers; `signals.md` — signals, memos and effects, and reaching
  them from outside; `components.md` — how a component is built and mounted, family by family;
  `styles.md` — tokens, classes, states, the shape of a selector, units, `var()`, `calc()`,
  `@keyframes`, what the compiler refuses and what it says; `overlays.md` — popovers, menus,
  dialogs, sheets, toasts and who owns them; `pictures.md` — images, icons and canvases;
  `rules.md` — what panics and what a caller must keep; `index.md` — the index.
- **The index.** `index.md` holds three tables — the facade, the components, the view — whose
  first cell is a name in backticks, then what it is for, the page of the catalog that shows it
  and its source file. The names are those the `pub use` and `pub type` lines export in
  `crates/morok/src/lib.rs:17-30`, `crates/components/src/lib.rs:91-135` (45 lines, 47 names) and
  `crates/components/src/view.rs:36-52` (17 names).
- **The CSS subset.** The tables: `Property { name, expectation, parse }`
  (`crates/css/src/property.rs:12-17`), 109 properties in eight families
  (`crates/css/src/property_syntax`), where `expectation` finishes "it takes …"; the seven states
  of `States::PSEUDO_CLASSES` (`crates/style/src/states.rs:16`); the two pseudo-elements of
  `crates/css/src/pseudo_element.rs`; the seven families of `TokenFamily::PREFIXES`
  (`crates/css/src/token_family.rs`). What no table holds — selectors, units, `var()`, `calc()`,
  `@keyframes`, the refusals pinned by `crates/css/tests/subset_errors.rs` — `styles.md` tells by
  hand, with `css` fences the guide test compiles.
- **Golden files.** `crates/css/tests/generated_tokens.rs:13` is the model: the test compares the
  committed file with the output and rewrites it under `MOROK_UPDATE_GOLDEN`.
- **Public means `pub`.** `unreachable_pub = "warn"` (`Cargo.toml`) and clippy denies warnings, so
  every item written `pub` is reachable from outside its crate. The rule therefore reads syntax
  with `syn` and needs no compiler; code inside a macro invocation (`quote!` in the generators of
  `morok-css`) is tokens to `syn`, not items. `syn` is pinned to `=3.0.9`: read the API of that
  version in the registry sources before designing on it.
- **Public crates.** The members whose manifest does not set `publish = false`: style, css,
  reactive, core, platform, components, morok. bench, catalog, teststand and xtask set it.
- **Gaps at HEAD.** A grep counts about 119 `pub` items with no doc line before them (core 54,
  style 26, components 17, css 13, reactive 4, platform 4, morok 1). The rule gives the exact
  list, with the items of public traits the grep did not count.
- Public documentation · a `pub` function, struct, enum, union, trait, type alias, constant or
  static without documentation is a finding → each of the eight kinds, undocumented and alone in
  a file, exits 1 and the report names its line.
- Public documentation · a `pub` function, constant or type of an inherent `impl` without
  documentation is a finding → an undocumented `pub fn` in `impl Thing` exits 1.
- Public documentation · an item of a `pub` trait without documentation is a finding → an
  undocumented method of a documented `pub trait` exits 1.
- Public documentation · documentation is a doc comment (`///`, `/** */`) or a `#[doc = …]`
  attribute, `include_str!` among them → the same item documented in each of the four forms
  exits 0.
- Public documentation · a field and a variant need none → a documented struct with an
  undocumented `pub` field and a documented enum with undocumented variants exit 0.
- Public documentation · an item that is not `pub` needs none, nor does an item of a trait
  implementation → undocumented private, `pub(crate)` and `pub(super)` items, an undocumented
  method of a `pub(crate)` trait and one in `impl Trait for Thing` exit 0.
- Public documentation · a `pub use` and a `pub mod` need none → both, undocumented, exit 0.
- Public documentation · code inside a macro invocation is not an item → a macro invocation
  whose tokens read `pub struct Generated;` exits 0.
- Public documentation · an item hidden with `#[doc(hidden)]` and an item compiled for tests only
  need none → an undocumented hidden `pub fn` and one inside `#[cfg(test)] mod tests` exit 0.
- Public documentation · the items of an inline module are checked as those of a file → an
  undocumented `pub fn` inside an inline `pub mod` exits 1.
- Public documentation · only the `src` of a public crate is read → of two crates with the same
  undocumented item the report names the one without `publish = false`, and names neither an
  item under `tests` nor one in `build.rs`.
- Public documentation · a file that does not parse is a finding → a file with an unclosed brace
  exits 1 and the report names it.
- Public documentation · the subcommand exits 0 when clean, 1 with findings printed as
  `path:line: reason`, 2 on a wrong command line → no directory argument exits 2.
- **Limit of the rule.** Modules are outside it: a `pub mod` is documented by its own file or not
  at all, and nothing checks which.
- **The rules of the guide test.** The index names exactly the exports, both ways; every file of
  `crates/morok/docs` is attached in `crates/morok/src/guide.rs`; no fence is tagged `ignore`;
  every `css` fence compiles as a stylesheet over the tokens of the kit; every path of the
  repository the guide names — text in backticks or a link that starts with `crates/`, `design/`
  or `docs/`, a line number dropped — exists; the overview leads to every chapter and the README
  to the overview.
- **Comment rules.** The scanner reads `.rs`, `.metal` and `.css` under `crates/`: a doc comment
  that writes phase, task or step before a number is a finding. Markdown is not scanned.
- **Clippy on the guide.** The pedantic group is on; if `doc_markdown` reads the attached chapters,
  the names of code in them go in backticks.
- **The check.** `crates/xtask/src/check.rs:14-18` lists the steps and runs every one even after
  one fails; a step that needs cargo goes through `cargo_succeeds`. `--doc` cannot share a cargo
  call with `--test`.
- **Cold read.** The last check is a reader. A reviewer spawned with no knowledge of the kit
  starts at `README.md` and may read whatever the guide leads to. The brief: an application with
  one window titled Notes — a search field, a list of five notes that shows those whose text
  holds what the field holds, and a button that empties the field — on the styles of the kit
  alone. The reviewer writes it as one Rust file and reports where the guide left it guessing.
  The phase compiles the file unchanged as an example of `morok`, then removes it. It passes when
  the file compiles and no guess is one the guide should have answered.
- **Lint at HEAD.** `cargo xtask check` exits 0 at `2795a7d`, the working tree unchanged before
  and after. `cargo xtask public-docs .` was not run at HEAD: the phase creates the subcommand,
  and by the grep it starts red — closing those findings is the work of the phase.

# Gameplan

<!-- 4 phases. The rule comes first so that the public items the later phases add are checked. -->

### Phase 1: public-docs — Every public item of the public crates is documented, and a rule of xtask holds it

- [ ] The rule of public documentation in `xtask`, on `syn` and `toml`: it reads the `src` of
  every member of a workspace whose manifest does not set `publish = false` and reports, through
  `Finding`, every item the rules of Knowledge ask documentation for.
- [ ] The subcommand `cargo xtask public-docs <workspace directory>` beside
  `comments <directory>`, with the exit codes of Knowledge; the usage line names it.
- [ ] `crates/xtask/tests/public_docs.rs`: one case for every rule, as Knowledge pairs them; the
  fixtures are written into `CARGO_TARGET_TMPDIR` during the test, as `comment_rules.rs` writes
  its own.
- [ ] Documentation for every item the rule reports in the seven public crates, by the convention
  of doc comments; no behaviour and no signature changes.
- [ ] A mutation proves the rule on the real tree: the doc comment of one public item removed
  turns `cargo xtask public-docs .` red with its line; the comment is put back.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p xtask --test public_docs
```
- The rule of public documentation holds each of its rules on a fixture (exit 0).
```timeout=900s
cargo xtask public-docs .
```
- Every public item of the seven public crates is documented (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and the comment rules are clean, the new doc comments among them (exit 0).

### Phase 2: css-reference — The reference of the CSS subset is rendered from the tables of the compiler (consumes phase 1: the rule of public documentation)

- [ ] `morok_css::subset_reference() -> String`: the reference as markdown, rendered from the
  tables alone — the properties family by family, each with what it takes; the states; the
  pseudo-elements with what a rule of theirs may set; the families of tokens. The list of the
  families gets one owner, which `property::named` and the reference both read.
- [ ] `crates/css/docs/css-subset.md`: the committed output, opened by a line that says it is
  generated and by which command.
- [ ] `crates/css/tests/subset_reference.rs`: the file is what the function returns;
  `MOROK_UPDATE_GOLDEN` rewrites it.
- [ ] A mutation proves the test: one `expectation` changed turns it red; it is changed back.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok-css --test subset_reference
```
- The committed reference is what the tables of the compiler render (exit 0).
```
grep -q flex-direction crates/css/docs/css-subset.md
```
- The reference lists the properties of the subset (exit 0).
```
grep -q focus-visible crates/css/docs/css-subset.md
```
- The reference lists the states of the subset (exit 0).
```timeout=900s
cargo xtask public-docs .
```
- The function that renders the reference is documented, as the rule asks of a public item (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and the comment rules are clean (exit 0).

### Phase 3: guide — The guide travels with the facade and compiles its own examples (consumes phase 2: the reference of the subset)

- [ ] The chapters of Knowledge in `crates/morok/docs/`, written from the sources and the
  catalog: each answers what Knowledge names for it, in the conventions of the documentation.
- [ ] `crates/morok/src/guide.rs`: the module `guide` of the facade, one documentation-only
  module per chapter, attached with `#[doc = include_str!(…)]`; the crate documentation of
  `crates/morok/src/lib.rs` leads to it.
- [ ] Every Rust fence is a doctest that compiles, on `Styles::of_kit()` where it builds an
  application; what needs the tables of an application names the files of the catalog instead.
- [ ] `index.md`: the three tables of Knowledge, a row for every exported name.
- [ ] `styles.md` leads to `crates/css/docs/css-subset.md` for the properties and tells by hand
  what no table holds, every example a `css` fence.
- [ ] `crates/morok/tests/guide.rs`, with `morok-css` as a dev-dependency: the rules of the guide
  test of Knowledge, a test for each; of the last rule, the part the overview owns.
- [ ] A mutation proves each rule of the test and is taken back: a row of the index removed, a
  row for a name nothing exports, a chapter left unattached, a fence tagged `ignore`, a property
  outside the subset in a `css` fence, a path that does not exist.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok --doc
```
- Every Rust fence of the guide compiles as a doctest of the facade (exit 0).
```timeout=900s
cargo test -p morok --test guide
```
- The index names exactly the exports, every chapter is attached, no fence is ignored, every `css` fence compiles and every named path exists (exit 0).
```
grep -q include_str crates/morok/src/guide.rs
```
- The chapters are attached to the documentation of the facade (exit 0).
```
grep -q css-subset.md crates/morok/docs/styles.md
```
- The chapter on styles leads to the rendered reference of the subset (exit 0).
```timeout=900s
cargo xtask public-docs .
```
- The public items of the public crates are still documented (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy and the comment rules are clean, the attached chapters among what clippy reads (exit 0).

### Phase 4: wire — The check holds the documentation to the code, and a reader starts at the README (consumes phases 1-3)

- [ ] `check::is_clean` gains the documentation checks as steps of their own: the rule of public
  documentation over the workspace, the test of the reference, the doctests of `morok` and the
  guide test.
- [ ] `README.md` at the root: what MOROK is and what it needs, how to run the catalog, and where
  the guide starts.
- [ ] The guide test gains the rest of its last rule: the README leads to the overview.
- [ ] `docs/MOROK-ARCHITECTURE.md`: where the guide lives and which checks hold it, in the
  section on the crate layout.
- [ ] A mutation proves the whole path: a file the guide names is renamed, `cargo xtask check`
  turns red at the step of the guide test, and the name is put back.
- [ ] The cold read of Knowledge.

**Done when (EXECUTABLE):**
```timeout=900s
cargo xtask check
```
- Formatting, clippy, the comment rules and the documentation checks are clean (exit 0).
```
grep -q public_docs crates/xtask/src/check.rs
```
- The check runs the rule of public documentation (exit 0).
```
grep -q subset_reference crates/xtask/src/check.rs
```
- The check runs the test of the reference of the subset (exit 0).
```
grep -q guide crates/xtask/src/check.rs
```
- The check runs the doctests and the test of the guide (exit 0).
```timeout=900s
cargo test -p morok --test guide
```
- The README leads to the overview and the overview to every chapter (exit 0).
```
grep -q crates/morok/docs README.md
```
- The README names where the guide lives (exit 0).
```
grep -q crates/morok/docs docs/MOROK-ARCHITECTURE.md
```
- The architecture names where the guide lives (exit 0).

**Done when (AGENT-JUDGED):** the cold read of Knowledge passes — a reviewer with no knowledge of the kit writes the Notes window from the guide alone, the file compiles unchanged as an example of `morok`, and no guess it reports is one the guide should have answered; agent-judged because what is judged is whether an agent can work from the guide alone, which no command decides.
