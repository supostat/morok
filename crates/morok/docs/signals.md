# Signals

The state of an application lives in a signal graph, `morok::reactive::Runtime`. It holds three
kinds of node, each reached by a handle that is `Copy`:

- A **signal** holds a value that is written from outside the graph: by a handler, or by the
  application.
- A **memo** holds a value computed from signals and other memos. It is computed when it is read,
  and again only after a value it read changed.
- An **effect** is a function that runs in the next flush, and again in the flush after a value it
  read changed.

A memo or an effect depends on exactly what its last run read: there is no list of dependencies
to keep.

```rust
use morok::reactive::Runtime;

let mut runtime: Runtime = Runtime::new();
let price = runtime.signal(4_u32);
let quantity = runtime.signal(3_u32);
let total = runtime.memo(move |runtime| runtime.get(price) * runtime.get(quantity));
assert_eq!(runtime.get(total), 12);

runtime.set(quantity, 5);
assert_eq!(runtime.get(total), 20);
```

| Method | What it does |
|---|---|
| `signal(value)` | Creates a signal. |
| `memo(compute)` | Creates a memo. A new value equal to the old one wakes nothing that depends on the memo. |
| `effect(run)` | Creates an effect. |
| `get(source)` | A copy of the value of a signal or a memo. |
| `with(source, read)` | Reads the value in place, without a copy. |
| `set(signal, value)` | Replaces the value. What depends on the signal becomes stale whether or not the new value differs. |
| `update(signal, change)` | Changes the value in place. |
| `untracked(read)` | Reads without making what runs right now depend on what is read. |

A value of a signal is any type that is `'static`. `get` asks it to be `Clone`; `with` asks
nothing more. The value of a memo is `PartialEq` as well: that is how a memo tells that nothing
changed. An item of `for_each` is `Clone`, and its key `PartialEq`.

## Writes and flushes

A write only marks what depends on the signal. The effects run in a **flush**: however many
signals were written since the last one, an effect runs once and sees values that agree with
each other.

```rust
use std::cell::RefCell;
use std::rc::Rc;

use morok::reactive::Runtime;

let mut runtime: Runtime = Runtime::new();
let name = runtime.signal(String::from("Ada"));
let seen = Rc::new(RefCell::new(Vec::new()));
let log = Rc::clone(&seen);
runtime.effect(move |runtime, _world| log.borrow_mut().push(runtime.get(name)));

runtime.flush(&mut ()).expect("no effect wakes itself");
runtime.set(name, String::from("Grace"));
runtime.set(name, String::from("Linus"));
runtime.flush(&mut ()).expect("no effect wakes itself");

assert_eq!(*seen.borrow(), ["Ada", "Linus"]);
```

**An application never flushes.** The kit flushes at the start of every frame, and a write asks
for a frame. So the path of every change is the same: a handler writes signals, the frame runs the
effects, the effects change the elements, the frame lays out and paints what changed.

The graph of an application is `morok::Runtime`: a `Runtime` whose effects are handed the surfaces
of the application. Every place the kit hands a `&mut Runtime<Surfaces>` to — a handler, `on_press`,
`Mount::runtime`, `Application::update` — is this one graph.

## In a component

A component creates its signals through `mount.runtime()` and hands them to the properties of its
elements ([Elements](elements.md#properties-a-value-or-a-signal)). A value derived from signals is
a memo:

```rust
use morok::components::view::Mount;
use morok::components::{Button, Checkbox};
use morok::core::ElementId;

fn agreement(mount: &mut Mount<'_>) -> ElementId {
    let has_read = mount.runtime().signal(false);
    let has_agreed = mount.runtime().signal(false);
    let cannot_go_on = mount
        .runtime()
        .memo(move |runtime| !(runtime.get(has_read) && runtime.get(has_agreed)));

    let read = Checkbox::new(has_read, "I have read the terms").mount(mount);
    let agreed = Checkbox::new(has_agreed, "I agree").mount(mount);
    let go_on = Button::new("Continue").primary().disabled(cannot_go_on).mount(mount);
    mount.element().child(read).child(agreed).child(go_on).id()
}
```

A component seldom writes an effect: binding a property creates one. When it does need to drive
an element from signals itself, `Mount::effect` is an effect that is handed the tree of the
window:

```rust
use morok::components::view::Mount;
use morok::core::ElementId;

fn title(mount: &mut Mount<'_>) -> ElementId {
    let name = mount.runtime().signal(String::from("Untitled"));
    let text = mount.text("").id();
    mount.effect(move |runtime, tree| {
        let title = format!("{} — Notes", runtime.get(name));
        tree.set_text(text, &title);
    });
    text
}
```

## A filtered list

What a list shows is a memo over the signals it depends on — here the notes and the query of a
search field — and `for_each` follows the memo. A button that empties the field writes the same
signal the field edits.

The example shows the signals alone: its column is a plain element. What gives a window its
background, its font and the room around its content is in
[Elements](elements.md#the-outermost-element-of-a-window).

```rust
use morok::components::view::Mount;
use morok::components::{Button, SearchField};
use morok::core::ElementId;

fn notes(mount: &mut Mount<'_>) -> ElementId {
    let notes = mount.runtime().signal(vec![
        String::from("Buy milk"),
        String::from("Call the bank"),
        String::from("Milk the cow"),
    ]);
    let query = mount.runtime().signal(String::new());
    let shown = mount.runtime().memo(move |runtime| {
        let query = runtime.get(query).to_lowercase();
        runtime.with(notes, |notes| {
            notes
                .iter()
                .filter(|note| note.to_lowercase().contains(&query))
                .cloned()
                .collect::<Vec<String>>()
        })
    });

    let field = SearchField::new(query)
        .placeholder("Search")
        .label("Search the notes")
        .mount(mount);
    let clear = Button::new("Clear")
        .on_press(move |runtime| runtime.set(query, String::new()))
        .mount(mount);
    let mut column = mount.element();
    column.child(field).child(clear);
    column.for_each(shown, |note| note.clone(), |mount, note| mount.text(note.as_str()).id());
    column.id()
}
```

## Who owns a node

A signal, memo or effect belongs to what was running when it was created, and goes with it:

- Created in a component, it lives as long as what the component built: until the window closes,
  or until the `show` or the `for_each` that mounted the component removes it.
- Created in a memo or an effect, it lives until the next run of that memo or effect.
- Created through `Application::update`, it lives as long as the application.

**A handle that outlived its node panics when it is used.** A signal shared by two components is
therefore created by what outlives both: their parent, or the application.

## From outside a component

`Application::update` hands the graph to code that is no component. It creates the signals
several windows share, before any window opens, and writes signals from a callback of the
application.

```rust,no_run
use morok::core::Size;
use morok::{Application, RendererError, Styles, WindowOptions};

fn main() -> Result<(), RendererError> {
    let application = Application::new(Styles::of_kit())?;
    let status = application.update(|runtime| runtime.signal(String::from("Waiting")));
    let options = WindowOptions {
        title: String::from("Status"),
        content_size: Size {
            width: 320.0,
            height: 120.0,
        },
        blur_behind: false,
    };
    let _window = application.open_window(&options, move |mount| mount.text(status).id());

    application.update(|runtime| runtime.set(status, String::from("Done")));
    application.run();
    Ok(())
}
```

`update` is not called from inside a handler or a component: they have the graph already.

## What to avoid

- An effect that writes a signal it reads wakes itself. The frame gives up after a number of runs
  and panics: `Runtime::flush` reports it as a `FlushCycle`, which the application of the kit
  turns into a panic. Derive the value with a memo instead.
- A memo that reads its own value while computing it panics.
- `set` always wakes what depends on the signal. A handler that may write the value the signal
  holds already compares first when the work downstream is costly; a memo in between stops an
  unchanged value by itself.
