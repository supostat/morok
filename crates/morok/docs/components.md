# Components

The components of the kit are in `morok::components`. Every one is built the same way:

1. `new` takes what the component cannot do without — most often the signal it shows.
2. Methods chained after it, each taking the component by value, give it the rest: `label`,
   `placeholder`, `disabled`, `on_press`.
3. `mount(mount)` builds its elements and returns the outermost as an `ElementId`, which the
   application puts into the tree like any element of its own.

```rust
use morok::components::view::Mount;
use morok::components::{Button, TextField};
use morok::core::ElementId;

fn sign_in(mount: &mut Mount<'_>) -> ElementId {
    let name = mount.runtime().signal(String::new());
    let is_nameless = mount.runtime().memo(move |runtime| runtime.get(name).is_empty());

    let field = TextField::new(name).placeholder("Name").label("Name").mount(mount);
    let button = Button::new("Sign in")
        .primary()
        .disabled(is_nameless)
        .on_press(move |runtime| runtime.set(name, String::new()))
        .mount(mount);
    mount.element().child(field).child(button).id()
}
```

Three rules hold for all of them:

- **A component that shows a value shows a signal and writes it back.** `TextField::new(name)`
  shows what `name` holds and writes what the user types to it; a write to `name` from anywhere
  else changes the field. There is no separate "on change": whatever needs the value reads the
  signal, in a memo or an effect.
- **`label` is what an assistive technology calls the component.** A component whose text names
  it already — a `Button`, a `Checkbox` — takes its label in `new`; one that shows no name of its
  own takes it through `label`.
- **A handler is given the signal graph**, `&mut Runtime<Surfaces>`, and writes signals.

Every component answers the pointer, the keyboard and an assistive technology; its doc comment
says how. The pages named below are in `crates/catalog/src/pages`.

## Buttons

`buttons_page.rs`.

| Component | Built with |
|---|---|
| `Button` | `Button::new(label)`, then `primary()`, `disabled(is_disabled)`, `on_press(handler)`. |
| `IconButton` | `IconButton::new(icon, label)`: an icon of `KitIcon` in place of a text. `toggled(is_on)` makes it a toggle that flips a `Signal<bool>`. |

## Text fields

`text_fields_page.rs`. Each edits a `Signal<String>`.

| Component | Built with |
|---|---|
| `TextField` | `TextField::new(value)`: one line. `placeholder`, `label`, `disabled`, `invalid(is_invalid)`, `on_submit(handler)` for Return. |
| `SearchField` | `SearchField::new(query)`: one line in a rounded box with a magnifier. `placeholder`, `label`, `disabled`, `on_submit`. |
| `TextArea` | `TextArea::new(value)`: lines that wrap and scroll. `placeholder`, `label`, `disabled`, `invalid`. |

## Controls

`controls_page.rs`.

| Component | Built with |
|---|---|
| `Checkbox` | `Checkbox::new(is_checked, label)` over a `Signal<bool>`. |
| `Switch` | `Switch::new(is_on, label)` over a `Signal<bool>`. |
| `RadioGroup` | `RadioGroup::new(selected)` over a `Signal<Value>`, then an `option(value, label)` for every radio button. |
| `SegmentedControl` | `SegmentedControl::new(selected)` over a `Signal<Value>`, then a `segment(value, label)` for every segment. |
| `Slider` | `Slider::new(value)` over a `Signal<f64>`, from zero to one in hundredths; `range(range, step)` gives it other numbers. |
| `ProgressBar` | `ProgressBar::determinate(done)`, from zero to one, or `ProgressBar::indeterminate()`. |
| `Spinner` | `Spinner::new()`. |

The value a `RadioGroup`, a `SegmentedControl`, a `Select`, `Tabs` or a `Sidebar` chooses is any
type that is `Clone + PartialEq`: most often an enum of the application.

```rust
use morok::components::view::Mount;
use morok::components::{RadioGroup, Slider, Switch};
use morok::core::ElementId;

#[derive(Clone, Copy, PartialEq)]
enum Quality {
    Draft,
    Final,
}

fn export_settings(mount: &mut Mount<'_>) -> ElementId {
    let quality = mount.runtime().signal(Quality::Draft);
    let scale = mount.runtime().signal(1.0);
    let keeps_metadata = mount.runtime().signal(true);

    let quality = RadioGroup::new(quality)
        .option(Quality::Draft, "Draft")
        .option(Quality::Final, "Final")
        .label("Quality")
        .mount(mount);
    let scale = Slider::new(scale).range(0.5..=4.0, 0.5).label("Scale").mount(mount);
    let metadata = Switch::new(keeps_metadata, "Keep metadata").mount(mount);
    mount.element().child(quality).child(scale).child(metadata).id()
}
```

A group with no options panics when it is mounted: a `RadioGroup`, a `SegmentedControl`, a
`Select`, `Tabs` and a `Sidebar` each need one at least.

## Select and combobox

`pickers_page.rs`.

| Component | Built with |
|---|---|
| `Select` | `Select::new(selected)` over a `Signal<Value>`, then an `option(value, name)` for every option. It shows the option whose value the signal holds. |
| `Combobox` | `Combobox::new(value)` over a `Signal<String>`, then an `option(name)` for every option. The text is the user's: it need not name an option. |

Both drop their list down in an overlay of their own; the application opens nothing.

## Collections

`collections_page.rs` and `table_page.rs`.

| Component | Built with |
|---|---|
| `List` | `List::new(count, label_of, selected)`: `count` rows, `label_of` writes the text of the row of an index into the string it is given, `selected` is a `Signal<Vec<usize>>` of the selected indices. `selection(RowSelection::Multiple)` lets the user select several. |
| `Tree` | `Tree::new(roots, expanded, selected)`: `roots` is a `Vec<TreeItem<Key>>`, `expanded` a `Signal<Vec<Key>>` of the open items, `selected` a `Signal<Option<Key>>`. |
| `Table` | `Table::new(row_count, columns, cell, selected, sort)`: `columns` is a `Signal<Vec<TableColumn<Key>>>` the user resizes and reorders, `cell` writes the text of a cell, `sort` a `Signal<Option<TableSort<Key>>>` the headers write. The application puts its rows in the order the sort asks for. |
| `Tabs` | `Tabs::new(selected)` over a `Signal<Value>`, then a `tab(value, label)` for every tab. What a tab shows is the application's: a `show` that follows the signal. |

A `List`, a `Tree` and a `Table` hold as many rows as there may be — a million in the catalog —
because only the rows in view have elements. That has two consequences:

- **They need a size.** Each is a view that scrolls inside the height its style gives it.
  The catalog sizes them with a rule of its stylesheet, `.page-row > .scroll-view` in
  `crates/catalog/styles/pages.css`; an application without stylesheets gives the element the
  component returned a height with `mount.build(list).inline(…)`.
- **The text of a row is written by its index**, by a function the component calls for the rows
  in view. `label_of` of a `List` is given no signal graph: a list whose rows change with a
  signal is rather a `for_each` over a memo ([Signals](signals.md#a-filtered-list)), which suits
  any list short enough to build whole. `cell` of a `Table` is given the graph, and a cell shows
  its text anew whenever a signal `cell` read changes.

```rust
use std::fmt::Write;

use morok::components::view::Mount;
use morok::components::{List, RowSelection};
use morok::core::ElementId;

fn documents(mount: &mut Mount<'_>) -> ElementId {
    let selected = mount.runtime().signal(vec![0]);
    List::new(
        10_000,
        |row, label| write!(label, "Document {}", row + 1).expect("a string takes what is written to it"),
        selected,
    )
    .selection(RowSelection::Multiple)
    .label("Documents")
    .mount(mount)
}
```

## The scaffolding of a window

`scaffolding_page.rs`; the window of the catalog itself, `crates/catalog/src/catalog_window.rs`,
puts all of them together.

| Component | Built with |
|---|---|
| `Toolbar` | `Toolbar::new()`, then its parts left to right: `title(text)`, `item(component)` for a control, `spacer()` for the room between. |
| `StatusBar` | `StatusBar::new()`, then `text(text)`, `command(chord, label, on_press)` and `spacer()`. |
| `Sidebar` | `Sidebar::new(selected)` over a `Signal<Value>`, then `heading(text)` and `item(SidebarItem::new(value, label))`; a `SidebarItem` takes an `icon` and a `detail`. |
| `SplitView` | `SplitView::new()`, then a `pane` for every pane: `SplitPane::sized(length, limits, content)` over a `Signal<f32>` the divider writes, and exactly one `SplitPane::flexible(content)` that takes the rest. `axis(SplitAxis::Vertical)` stacks the panes. |
| `ScrollView` | `ScrollView::new(content)`: a region whose content scrolls under overlay scroll bars. Whoever uses it gives it its size. |

Where a part is built by a component of the application — the `item` of a `Toolbar`, the content
of a `SplitPane` or a `ScrollView` — it is handed as a closure that the scaffold calls with its
own `Mount`:

```rust
use morok::components::view::Mount;
use morok::components::{IconButton, KitIcon, SplitPane, SplitView, Toolbar};
use morok::core::ElementId;

fn workbench(mount: &mut Mount<'_>) -> ElementId {
    let sidebar_width = mount.runtime().signal(220.0);
    let toolbar = Toolbar::new()
        .title("Workbench")
        .spacer()
        .item(|mount| IconButton::new(KitIcon::Plus, "New document").mount(mount))
        .label("Workbench")
        .mount(mount);
    let panes = SplitView::new()
        .pane(SplitPane::sized(sidebar_width, 160.0..=320.0, |mount| {
            mount.text("Places").id()
        }))
        .pane(SplitPane::flexible(|mount| mount.text("The document").id()))
        .label("Workbench")
        .mount(mount);
    mount.element().child(toolbar).child(panes).id()
}
```

A `SplitView` fills the box it is put in, so that box needs a height: in the catalog the window
is a column as tall as the surface and the split view grows to fill it, `.catalog-window` and
`.catalog-body` in `crates/catalog/styles/catalog.css`.

## Overlays and pictures

`Popover`, `Tooltip`, `Menu`, `ContextMenu`, `Dialog`, `Sheet` and `ToastStack` are in
[Overlays](overlays.md); `Image` and the icons of `KitIcon` in [Pictures](pictures.md).

## A component of the application

A component of an application is a function, as in every example above. One with options is a
struct with a `mount` method, as the components of the kit are; any of them is a model, and
`crates/components/src/button.rs` the shortest. Inside, a component is built of elements, classes
and handlers ([Elements](elements.md)) and of other components.
