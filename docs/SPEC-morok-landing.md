<!--
  SPEC-morok-landing.md — the landing page of MOROK for GitHub Pages, written by the etalon
  `design/pages/landing/landing.html`: a folder `site/` that is published as it lies, its pictures
  frames the kit draws, its dark tokens rendered from the tokens of the kit, and a check of xtask
  that holds the folder whole. 4 phases.

  DECISION BOUNDARY (fan of 2026-11-27, option B chosen on the recommendation): the finished site
  lies in `site/` and a workflow uploads the folder with no build; the files derived from the
  tokens are committed and a check holds them. Rejected: A — a command assembles the site on the
  runner: nothing derived lies in git, but the page does not open as a file without a build and
  the runner needs the Rust toolchain (whether xtask builds on Linux was not checked by a run).
  C — no workflow, a local command fills and pushes a `gh-pages` branch: the frames are always
  fresh and never in `main`, but a tool commits and pushes, the publication is tied to one
  machine, and no test covers the push.
-->

# Baseline

- Prior spec: `docs/SPEC-morok-option-column.md` — its one phase closed (run 61704c5c), committed
  as `593051e`; the design of the page followed as `deb1a7b`, which is HEAD, pushed to
  `origin/main`.
- Prior spec-hash: sha256:7a885efaf860cbe16ca636a25131bbfd6b709215ac54a7e569c57fb03f92fe47
- Exists and is not touched: the kit — its behaviour, its stylesheets and every public signature;
  the etalons of the stand, its thresholds and the bench budgets; `design/system/tokens.css`; the
  etalon of the page and its drafts under `design/pages/landing`.
- Exists: the etalon `design/pages/landing/landing.html`, confirmed and fixed;
  `design/system/tokens-dark.css`, written by hand; a stand that draws the frames of the kit only
  to compare them with the etalons of Chrome.
- Missing: no `site/`, no `.github/`, no command that writes a frame of the kit to a file.
- Out of scope: a favicon, a social preview, analytics and a custom domain; a link to the site
  from `README.md`; the light scheme; any other page; a test that keeps the frames fresh and a
  check of the numbers the page states — the first was declined at the design, the second is
  held by review.
- The owner's: turning Pages on with GitHub Actions as its source. Pages of a private repository
  needs a paid plan; whether the repository is public was not checked.
- Version: no release automation — the workflow this spec adds deploys the site and releases
  nothing — and nothing is published: the workspace stays at `0.1.0`, with no version phase.
- Rationale of the choice — staged decision note (id after accept).

# Stack

- No new crates in the workspace: `xtask` takes `cssparser` from the workspace dependencies.
- `morok-teststand`: the command `frames <list>`; `crates/teststand/src/frame_list.rs` (new) — the
  list and its entries; `crates/teststand/src/frames.rs` (new) — the command;
  `crates/teststand/tests/frames.rs` (new).
- `xtask`: the subcommands `site-tokens <workspace directory>` and `site <workspace directory>`;
  `dark_tokens.rs` — the dark block of a tokens file and the mirror it renders; `site_tokens.rs` —
  the files derived from the tokens; `site_check.rs` — the check; the readers the check needs —
  the start tags of a page, the list of frames, the size of a PNG — each in a file of its own;
  `crates/xtask/tests/site.rs` (new); `check.rs` runs the check of the site.
- `site/` (new): `index.html`, `landing.css`, `frames.toml`, six pictures under `frames/`, and the
  derived `tokens.css` and `tokens-dark.css`.
- `design/system/tokens-dark.css` — rendered from now on; `design/system/DESIGN.md`.
- `.github/workflows/pages.yml` (new).
- `docs/MOROK-ARCHITECTURE.md`.

# Conventions

- Every convention of the earlier specs holds: etalons come only from `etalons render`; the stand
  thresholds and the bench budgets do not change.
- The folder `site/` is published as it lies: nothing in it is built on the runner, and nothing
  outside it is published.
- A picture of the page is a frame the stand wrote from `site/frames.toml`: none is drawn by
  hand, taken from a screen or copied from the etalons of the stand.
- The files derived from the tokens are never edited by hand: `cargo xtask site-tokens .` writes
  them.
- The page takes every color, size and duration from the tokens, as the etalon does; raw values
  lie in the two token files of the folder alone.
- The page and its stylesheet are English. A comment in them says why and never what, and names
  no fixture, state, etalon or decision of the design.
- Motion is CSS alone. The page holds one script: the one that shows the text of a picture that
  did not load.
- Every reference inside the folder is relative: the site is served under the path of the
  repository.
- Every action of the workflow is pinned to a full commit SHA, with the release it belongs to
  named on the same line.

# Knowledge

- **The etalon.** `design/pages/landing/landing.html` is one file: its styles inline, three
  fixtures and three states behind a switcher. The page is its fixture `typical` in the state
  `loaded` — the texts word for word, the sections in its order, the links as they are.
- **What the page leaves out.** The switcher and its script; the two `meta` of the manifest; the
  attributes `data-fixture`, `data-state`, `data-active-fixture`, `data-active-state` and the
  rules that read them; the texts of `minimal` and `extreme`, the tile of the select among them;
  the comments of the etalon, which annotate decisions of the design.
- **Where the styles lie.** An etalon is one file, so its styles are inline. The page holds them
  in `site/landing.css` and links three stylesheets in this order: `tokens.css`,
  `tokens-dark.css`, `landing.css`.
- **The states on the page.** `loaded` is the page. `still` is the rule under
  `prefers-reduced-motion: reduce` the etalon already holds, and a browser without scroll
  timelines stands in the final state by the `@supports` of the etalon. `no-pictures` becomes
  behaviour: a picture that fails to load is hidden and the text beside it (`.fallback`) shows in
  its frame. A stylesheet cannot see a failed load, so a script of a few lines in the head
  listens for `error` in the capture phase and marks the frame of the picture; with scripts off
  the browser shows the `alt` of the picture.
- **The pictures of the etalon are Chrome's.** `etalons::render`
  (`crates/teststand/src/etalons.rs`) draws every case in the pinned Chrome and stores that: the
  PNG files under `crates/teststand/etalons` are what Chrome drew, and the etalon of the page
  shows them as stand-ins. The page shows what the kit draws of the same cases; `check <suite>`
  holds the two within the thresholds of the stand, so an eye sees the same picture.
- **How the kit draws a case.** `MorokSide::new()`, then
  `render_case(&CompiledSuite, &Case) -> CaseRendering` (`crates/teststand/src/morok_side.rs`):
  the frame under each color scheme, drawn offscreen through Metal at `DEVICE_SCALE` 2 on a
  transparent surface. `suites::named(name)` gives a suite, `CompiledSuite::compile` compiles it,
  `frame_png::encode` (`pub(crate)`) writes a frame as a PNG. No window opens, but Metal is
  needed: the frames are written on a Mac and committed.
- **The six frames.** All under the dark scheme: `screens`/`file-manager` 1800×840,
  `screens`/`disk-map` 1800×880, `overlays`/`menu` 600×440, `table`/`table-states` 904×744,
  `button`/`primary-states` 1216×152, `text-fields`/`text-field-states` 1432×200 — the sizes of
  the etalons of those cases, which a frame of the kit shares.
- **The list of frames.** `site/frames.toml` is an array of tables `frame`, each with `suite`,
  `case`, `color_scheme` (`light` or `dark`, as `ColorScheme` displays itself) and `file`, a path
  under the directory of the list. The stand writes from it; the check of the site reads the
  names of its files.
- Frames · every entry is written as a PNG at its `file` under the directory of the list, the
  directories of the path created → a list of one entry whose file lies two directories deep
  leaves a PNG there, of the size of its case at the scale of the stand.
- Frames · a frame is what the kit draws of the case under the scheme its entry names → a list
  that names one case under `light` and under `dark` leaves two files that differ, and each
  matches, by the comparator of the stand at its thresholds, what `MorokSide` draws under its
  scheme.
- Frames · an entry that names no suite, no case of its suite or no color scheme is refused
  before anything is drawn → each of the three, second in a list after a sound entry, exits 2,
  the message names what is unknown, and the file of the first entry is not written.
- Frames · a `file` that is absolute or climbs out of the directory of the list is refused → a
  file `../out.png` and an absolute one each exit 2 and write nothing.
- Frames · a list that cannot be read as such — no file, a key missing, a key unknown — is
  refused → a list with a misspelt key exits 2 and the message names the list.
- Frames · the command exits 0 and prints how many frames it wrote, and a wrong command line
  exits 2 with the usage → a list of two entries prints `2 frames written`; `frames` with no list
  exits 2.
- **The tokens file.** `design/system/tokens.css` holds two blocks — `:root` and `:root` inside
  `@media (prefers-color-scheme: dark)` — and the compiler of the kit refuses anything else in
  it. A block holds custom properties alone, and the dark block repeats every token of the light
  one. The reader of the compiler, `crates/css/src/tokens_stylesheet.rs`, is `pub(crate)`: it is
  the model for reading the dark block with `cssparser`, not a function to call. `cssparser` is
  pinned to `=0.38.0`: read the API of that version in the registry sources before designing on
  it.
- **The mirror.** What the dark block renders: a line that says the file is generated and by
  which command, then one `:root` rule that holds `color-scheme: dark` and every declaration of
  the dark block in its order, each value as it is written. It is written to two places,
  `design/system/tokens-dark.css` for the etalons and `site/tokens-dark.css` for the page; the
  third derived file, `site/tokens.css`, is `design/system/tokens.css` byte for byte. The mirror
  written by hand held the colors and the shadows alone; the rendered one holds every token of
  the block, and the computed values stay the same, because the two blocks agree on the rest.
- **Why a command and a check, not a golden test.** The generated files of `morok-css` are held
  by tests that call a function of the library and rewrite under `MOROK_UPDATE_GOLDEN`. `xtask`
  is a binary: its tests run the binary. A derived file that fell behind is one more finding of
  the check that holds the rest of the folder, and one command writes all three.
- **The checks of xtask.** A check is `is_clean(&Path) -> bool` over `Finding { path, line,
  reason }`, printed by `finding_report::is_clean` as `path:line: reason`
  (`crates/xtask/src/public_docs_check.rs` is the model). A subcommand exits 0 when clean, 1 with
  findings, 2 on a wrong command line. The tests run the binary through `CARGO_BIN_EXE_xtask` on
  fixtures they write under `CARGO_TARGET_TMPDIR` (`crates/xtask/tests/public_docs.rs`).
- **Reading the page.** The workspace has no reader of HTML and takes none: the check reads the
  start tags of the page and their attributes in double quotes, and skips comments and the text
  of `script` and `style` elements. A local reference is the `src` of an `img` or a `script` and
  the `href` of a `link` whose value has no scheme and does not begin with `#`. A link into the
  repository is an `href` that begins with the `repository` of the workspace manifest followed
  by `/blob/main/`; what follows is a path from the root of the workspace.
- **The size of a PNG.** Its width and height are the two big-endian numbers of four bytes at
  offsets 16 and 20; the check reads those and decodes nothing.
- Site · `design/system/tokens-dark.css` is what the dark block of `design/system/tokens.css`
  renders → a value of the dark block changed after the files were written exits 1 and the
  report names the file.
- Site · `site/tokens-dark.css` is the same rendering → the file edited by hand exits 1 and the
  report names it.
- Site · `site/tokens.css` is `design/system/tokens.css` byte for byte → a value of the light
  block changed after the files were written exits 1 and the report names `site/tokens.css`.
- Site · a derived file that is missing is a finding → `site/tokens.css` removed exits 1 and the
  report names it.
- Site · a tokens file without a dark block renders nothing and is a finding → a tokens file of
  one `:root` block exits 1 and the report names `design/system/tokens.css`.
- Site · `site-tokens` writes the three files and creates the folder → on a workspace that holds
  `design/system/tokens.css` alone the command exits 0, the mirror is the text Knowledge
  describes for those tokens, and `site` then finds the token files current.
- Site · a subcommand exits 0 when clean, 1 with findings printed as `path:line: reason`, 2 on a
  wrong command line → `site` and `site-tokens` with no directory each exit 2.
- Site · the page and the list of frames exist → a folder without `index.html` exits 1 and the
  report names the page.
- Site · a local reference of the page names a file under the folder → an `img` whose file is
  missing, a `link` whose stylesheet is missing, and a `link` to
  `../design/system/tokens.css`, which exists outside the folder, each exit 1 with the line of
  the tag.
- Site · a picture of the page is a frame the list names, and a frame the list names is a file →
  an `img` whose file exists and is not in the list exits 1; an entry whose file is missing
  exits 1 and the report names the list.
- Site · the `width` and the `height` of a picture are those of its PNG → an `img` of a PNG four
  wide and two high declared three high exits 1, and so does an `img` with no `width`.
- Site · a link into the repository names a path the workspace holds → an `a` to a file under
  `/blob/main/` that does not exist exits 1 with the line of the tag.
- Site · a comment and the text of a `script` or a `style` are not read → an `img` of a missing
  file inside a comment, and one written inside the text of a `script`, exit 0.
- Site · a whole folder is clean → the fixture every case starts from exits 0.
- **Limits of the check.** It does not run the page: a reference a script builds and a URL inside
  a stylesheet are outside it. An attribute written without double quotes is not read: the page
  writes every attribute in double quotes. It does not search `landing.css` for raw values, and
  it does not compare the claims of the page — 109 properties and 7 states, Rust 1.94, macOS 26 —
  with the code: those are held by the acceptance against the etalon and by review. A frame is
  not compared with what the kit draws now: the frames are written again by hand, with the
  command.
- **Clippy in tests.** The pedantic group denies a strict comparison of `f32`: a test compares
  sizes as whole numbers or as the types of the geometry.
- **Pages.** A workflow that deploys with no build: it runs on a push to `main` that changes
  `site/**` or the workflow itself, and by hand; its permissions are `contents: read`,
  `pages: write` and `id-token: write`; one `concurrency` group keeps two deployments from
  running at once; one job in the environment `github-pages` checks the repository out, uploads
  `site` with `actions/upload-pages-artifact` and deploys with `actions/deploy-pages`. An
  uploaded artifact is served as it is: Jekyll runs only for a branch source. The release of
  each action and its commit are read from its repository when the workflow is written, never
  recalled.
- **Limit of the workflow.** Nothing in the repository runs a workflow. The gate proves that the
  file names the folder and the actions; its first run on GitHub is the proof, and it needs
  Pages turned on by the owner. The boundary of the last phase is the point of acceptance by a
  person.
- **Acceptance against the etalon.** A person opens `site/index.html` as a file beside the etalon
  switched to `typical` and `loaded`, and compares.
  Sections: the navigation, the statement with its badge, lead and two actions, the window, the
  three facts, five tiles of components, the pair of code, the four lines of what the kit is
  not, the last panel and the foot — in this order, with the texts of the etalon.
  States: the page as loaded; a picture whose file is renamed shows its text in its frame; with
  reduced motion nothing moves and nothing is missing; without scroll timelines the page stands
  in its final state.
  Accent: the accent color is where the etalon has it — the primary action, the first word of
  the badge, the names of components in the code and the highlight of the shared class, the line
  through what the kit is not, the one light of the scene and the glow of the last panel, the
  border of a tile under the pointer — and nowhere else; a screen holds one primary action.
  Units: every color, size and duration is a token or a multiple of one; the hairlines of one
  pixel are those of the etalon.
- **Lint at HEAD.** `cargo xtask check` exits 0 at `deb1a7b`, the working tree unchanged before
  and after. `cargo xtask site .` was not run at HEAD: the phase creates the subcommand.

# Gameplan

<!-- 4 phases. The frames come first: drawing through Metal is the one part that can fail for a reason outside the code. -->

### Phase 1: kit-frames — The stand writes the frames of the kit that a list names

- [ ] The list of frames in a file of its own: read from TOML with unknown keys refused, every
  entry resolved against the suites of the stand and the directory of the list before anything
  is drawn, by the rules of Knowledge.
- [ ] The command in a module of its own beside `etalons`: for every entry the frame `MorokSide`
  draws of the case under the scheme, written as a PNG; a suite named by several entries is
  compiled once.
- [ ] `morok-teststand frames <list>` in `main.rs`, with the exit codes and the printed line of
  Knowledge; the usage line names it.
- [ ] `crates/teststand/tests/frames.rs`: one case for every rule of the command, as Knowledge
  pairs them; the lists are written under `CARGO_TARGET_TMPDIR`.
- [ ] A mutation proves the scheme is read: the command made to write the light frame whatever
  the entry says turns the case of the two schemes red; it is taken back.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p morok-teststand --test frames
```
- The command writes the frames a list names and refuses a list it cannot serve (exit 0).
```
grep -q frames crates/teststand/src/main.rs
```
- The stand has the command (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy, the comment rules and the documentation checks are clean (exit 0).

### Phase 2: site-tokens — The token files of a page are rendered from the tokens of the kit (order-only after phase 1)

- [ ] The dark block of a tokens file read with `cssparser`, and the mirror it renders, as
  Knowledge describes both; a file without the block is an error the caller reports.
- [ ] `cargo xtask site-tokens <workspace directory>`: writes the three derived files, creating
  the folder of the site.
- [ ] `cargo xtask site <workspace directory>`: the check, with the rules of Knowledge that
  concern the token files and the exit codes; the usage line names both subcommands.
- [ ] `crates/xtask/tests/site.rs`: one case for each of those rules, as Knowledge pairs them.
- [ ] The three files written on this workspace with the command: the mirror of
  `design/system` rendered in place of the one written by hand, and the two files of `site/`.
- [ ] A mutation proves the check on the real tree: one value of the dark block changed and
  nothing written again turns `cargo xtask site .` red with both mirrors named; it is changed
  back.

**Done when (EXECUTABLE):**
```timeout=900s
cargo test -p xtask --test site
```
- The check holds each rule of the token files on a fixture, and the command writes what the check accepts (exit 0).
```timeout=900s
cargo xtask site .
```
- The token files of this workspace are what its tokens render (exit 0).
```
grep -q color-scheme site/tokens-dark.css
```
- The mirror of the site is written and declares the dark scheme (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy, the comment rules and the documentation checks are clean (exit 0).

### Phase 3: page — The page of the etalon lies in the folder of the site, its pictures frames of the kit (consumes phases 1-2: the command that writes frames and the check of the site)

- [ ] `site/frames.toml`: the six frames of Knowledge; the pictures under `site/frames/` written
  from it with the command of the stand.
- [ ] `site/index.html` and `site/landing.css`: the page of the etalon as Knowledge takes it —
  what it keeps, what it leaves out, where its styles lie — every `img` with the size of its
  frame.
- [ ] The script that shows the text of a picture that did not load, and the rule that hides the
  picture; no other script.
- [ ] The check gains the rules of Knowledge that concern the page, the list and the links, with
  the readers they need; `crates/xtask/tests/site.rs` gains a case for each.
- [ ] A mutation proves the check on the real tree: a picture of the page renamed turns
  `cargo xtask site .` red with the line of its tag; the name is put back.

**Done when (EXECUTABLE):**
```timeout=900s
cargo run -p morok-teststand -- frames site/frames.toml
```
- The stand draws the six frames of the page from its list (exit 0).
```timeout=900s
cargo xtask site .
```
- The folder is whole: every reference, picture, size and link of the page holds, and the token files are current (exit 0).
```timeout=900s
cargo test -p xtask --test site
```
- The check holds each of its rules on a fixture (exit 0).
```
grep -q frames/file-manager.png site/index.html
```
- The page shows the frame of the file manager (exit 0).
```
grep -q prefers-reduced-motion site/landing.css
```
- The stylesheet keeps the still form of the page (exit 0).
```timeout=900s
cargo xtask check
```
- Formatting, clippy, the comment rules and the documentation checks are clean (exit 0).

### Phase 4: wire — The check of the project holds the site and a workflow publishes its folder (consumes phases 1-3)

- [ ] `check::is_clean` gains the check of the site over the workspace as a step of its own.
- [ ] `.github/workflows/pages.yml`, as Knowledge lays it out, every action pinned by the
  convention.
- [ ] `design/system/DESIGN.md`: the mirror is rendered, by which command, and what holds it.
- [ ] `docs/MOROK-ARCHITECTURE.md`: the command `frames` in the section on the test stand; where
  the site lies, what writes its pictures and its token files and what holds the folder, in the
  section on platform and publishing.
- [ ] A mutation proves the whole path: a picture of the page renamed turns `cargo xtask check`
  red at the step of the site; the name is put back.

**Done when (EXECUTABLE):**
```timeout=900s
cargo xtask check
```
- Every step is clean on the real tree, the check of the site among them (exit 0).
```
grep -q site_check crates/xtask/src/check.rs
```
- The check of the project runs the check of the site (exit 0).
```
grep -q upload-pages-artifact .github/workflows/pages.yml
```
- The workflow uploads the folder as the artifact of Pages (exit 0).
```
grep -q deploy-pages .github/workflows/pages.yml
```
- The workflow deploys the artifact (exit 0).
```
grep -q site-tokens design/system/DESIGN.md
```
- The rules of the pages name the command that renders the mirror (exit 0).
```
grep -q site/frames.toml docs/MOROK-ARCHITECTURE.md
```
- The architecture names the list the pictures of the site are written from (exit 0).
