# Elements

A component is a function `(&mut Mount<'_>) -> ElementId`. It is called once, when it is mounted:
it creates its signals, builds its elements and returns the outermost of them. Whoever mounted it
puts that element into the tree.

```rust
use morok::components::view::Mount;
use morok::core::ElementId;

fn greeting(mount: &mut Mount<'_>) -> ElementId {
    let title = mount.text("Notes").id();
    let hint = mount.text("Nothing here yet").id();
    mount.element().child(title).child(hint).id()
}
```

## What a `Mount` creates

Every method creates one element and returns an `ElementBuilder` on it.

| Method | The element |
|---|---|
| `element()` | A box that lays out its children. |
| `text(text)` | A run of text. |
| `scroll()` | A box whose content scrolls inside it. The component `ScrollView` adds the scroll bars. |
| `text_input()`, `text_area()` | Text the user edits, on one line or in lines. The components `TextField` and `TextArea` add the box, the focus ring and the signal. |
| `kit_icon(icon)` | An icon of the kit. |
| `image(image)` | A picture the application added. |
| `canvas(canvas)` | A region the application draws itself. |

Three more methods build on an element that exists already: `surface_root()` on the root of the
window, whose properties hold for the whole window, and `build(element)` on any element by its
ID. `runtime()` is the signal graph, `effect(…)` an effect that is handed the tree
([Signals](signals.md)), and `tree()` the tree itself, for a component that drives elements
directly.

## The builder

A call of the builder gives the element a property, a child or a handler, and returns the builder
again. `id()` ends the chain with the `ElementId`.

The builder borrows the `Mount`, so **children are built first and the parent last**: every chain
ends in `id()` before the next begins, as in the example above. A parent that is given children in
a loop is held in a variable:

```rust
use morok::components::view::Mount;
use morok::core::ElementId;

fn lines(mount: &mut Mount<'_>, texts: &[&str]) -> ElementId {
    let children: Vec<ElementId> = texts.iter().map(|text| mount.text(*text).id()).collect();
    let mut column = mount.element();
    for child in children {
        column.child(child);
    }
    column.id()
}
```

## Properties: a value or a signal

Where a method takes `impl Into<Property<T>>`, it takes either a `T`, which stays, or a
`Signal<T>` or `Memo<T>`, which the property follows from then on: the kit creates the effect that
writes the property of this very element when the signal changes. A text takes a `&str` as well.

```rust
use morok::components::view::Mount;
use morok::core::ElementId;
use morok::style::States;

fn status(mount: &mut Mount<'_>) -> ElementId {
    let unread = mount.runtime().signal(3_u32);
    let label = mount
        .runtime()
        .memo(move |runtime| format!("{} unread", runtime.get(unread)));
    let is_empty = mount.runtime().memo(move |runtime| runtime.get(unread) == 0);
    mount.text(label).state(States::DISABLED, is_empty).id()
}
```

| Method | What it gives the element |
|---|---|
| `class(class)` | A class, for as long as the element lives. |
| `class_when(class, is_on)` | A class, for as long as `is_on` holds. |
| `state(states, is_on)` | The states, matched by `:disabled`, `:checked` and the others in a stylesheet, for as long as `is_on` holds. The tree drives `:hover`, `:active` and the focus states itself. |
| `inline(declaration)` | One property of its style, over every rule of the style sheet. |
| `placeholder(text)` | What an edited text shows while it is empty. |
| `focusable(focusability)` | Whether it takes focus and whether the Tab key stops on it. |
| `accessible_role(role)`, `accessible_name(name)` and the other `accessible_…` methods | Its node in the accessibility tree. The role comes first: the others panic on an element without one. |

## Classes

A class is a value of `morok::style::Class`: a field of the `CLASSES` table the build generated
from the stylesheets of the application ([An application](application.md#styles-of-its-own)).
`crates/catalog/src/page_layout.rs` shows elements built with classes; `.page-row` and the others
are declared in `crates/catalog/styles/pages.css`.

The classes of the components of the kit are in `morok::components::styles::CLASSES`. They are
what the components build themselves with, and their names are not a contract: an application
styles its own elements with its own classes.

## Inline declarations

An application without stylesheets of its own lays out its elements with `inline`. A
`Declaration` is one longhand property with its value: the enum `morok::style::Declaration` has a
variant for every longhand of the subset. A keyword is a variant of its enum; a color is a token
of `morok::components::tokens::TOKENS`.

```rust
use morok::components::tokens::TOKENS;
use morok::components::view::Mount;
use morok::core::ElementId;
use morok::style::{ColorValue, Declaration, Display, FlexDirection, ItemAlignment};

fn column(mount: &mut Mount<'_>, children: &[ElementId]) -> ElementId {
    let mut column = mount.element();
    column
        .inline(Declaration::Display(Display::Flex))
        .inline(Declaration::FlexDirection(FlexDirection::Column))
        .inline(Declaration::AlignItems(ItemAlignment::Start))
        .inline(Declaration::BackgroundColor(ColorValue::Token(TOKENS.color_surface)))
        .inline(Declaration::Color(ColorValue::Token(TOKENS.color_text)));
    for child in children {
        column.child(*child);
    }
    column.id()
}
```

A length is written in full: pixels, a fraction of the percentage basis and token terms. A
function of the application keeps that in one place.

```rust
use std::borrow::Cow;

use morok::components::tokens::TOKENS;
use morok::style::{Declaration, LengthPercentage, LengthPercentageAuto, Pixels, Side, Token, TokenTerm};

/// A length of so many CSS pixels.
fn pixels(pixels: f32) -> LengthPercentage {
    LengthPercentage {
        pixels: Pixels(pixels),
        fraction: None,
        tokens: Cow::Borrowed(&[]),
    }
}

/// The length a space token holds under the theme of the window.
fn space(token: Token<Pixels>) -> LengthPercentage {
    LengthPercentage {
        pixels: Pixels(0.0),
        fraction: None,
        tokens: Cow::Owned(vec![TokenTerm { token, factor: 1.0 }]),
    }
}

/// A box as wide as its parent and 240 pixels high, padded at the top, its rows 12 pixels apart.
fn panel() -> [Declaration; 4] {
    [
        Declaration::Width(LengthPercentageAuto::LengthPercentage(LengthPercentage {
            pixels: Pixels(0.0),
            fraction: Some(1.0),
            tokens: Cow::Borrowed(&[]),
        })),
        Declaration::Height(LengthPercentageAuto::LengthPercentage(pixels(240.0))),
        Declaration::Padding(Side::Top, space(TOKENS.space_4)),
        Declaration::RowGap(pixels(12.0)),
    ]
}
```

What an element looks like before any declaration reaches it:

- `display: block`, no margin, no padding, `box-sizing: border-box`.
- A transparent background, the root of the window included: an application that wants a
  background gives one to its outermost element.
- Text in the font of the system at 16 pixels, in black. The color and the properties of text
  pass from an element to its children, so one `Declaration::Color` on the outermost element
  colors every text inside it — and the texts of the components carry their own.

## Children that come and go

`child` appends an element for good. Two methods append children that follow a signal; both put
them after the children the element has at the moment of the call.

`show(when, content)` keeps a child there while `when` holds: `content` is called when it comes
to hold, and what it built is removed, with the signals and the effects it created, when it holds
no longer.

`for_each(items, key, item)` keeps a child for every item of a list, in the order of the list.
`key` tells the items apart: `item` is called once for a key that comes, an item whose key stays
keeps its element wherever it moves, and the element of a key that goes is removed. What changes
inside an item that keeps its key is not shown by itself: the item holds signals for it.

```rust
use morok::components::view::Mount;
use morok::core::ElementId;

#[derive(Clone)]
struct Note {
    id: u32,
    text: String,
}

fn notes(mount: &mut Mount<'_>) -> ElementId {
    let notes = mount.runtime().signal(vec![
        Note {
            id: 1,
            text: String::from("Buy milk"),
        },
        Note {
            id: 2,
            text: String::from("Call the bank"),
        },
    ]);
    let is_empty = mount
        .runtime()
        .memo(move |runtime| runtime.with(notes, |notes| notes.is_empty()));
    let mut list = mount.element();
    list.for_each(notes, |note| note.id, |mount, note| mount.text(note.text.as_str()).id());
    list.show(is_empty, |mount| mount.text("No notes").id());
    list.id()
}
```

A list filtered by what a field holds is a memo over the two signals, handed to `for_each`:
[Signals](signals.md#a-filtered-list) has it.

## Handlers

`on_event` names what hears of the events of the element: what the pointer does to it, the keys
that go down and up while focus is on it or inside it, and what an assistive technology asks of
it. The handler is given the signal graph and the event, writes signals, and answers
`Propagation::Stop` for an event it used or `Propagation::Continue` for one that goes on to the
element around this one. An element has one handler: a second `on_event` replaces the first.

```rust
use morok::components::view::{ElementEvent, Mount};
use morok::core::{ElementId, Focusability, Key, KeyPhase, PointerEventKind, Propagation};

fn counter(mount: &mut Mount<'_>) -> ElementId {
    let count = mount.runtime().signal(0_u32);
    let label = mount
        .runtime()
        .memo(move |runtime| format!("clicked {} times", runtime.get(count)));
    mount
        .text(label)
        .focusable(Focusability::TabStop)
        .on_event(move |runtime, event| match event {
            ElementEvent::Pointer(pointer) if matches!(pointer.kind, PointerEventKind::Clicked { .. }) => {
                runtime.update(count, |count| *count += 1);
                Propagation::Stop
            }
            ElementEvent::Key(key) if key.phase == KeyPhase::Pressed && key.key == Key::Space => {
                runtime.update(count, |count| *count += 1);
                Propagation::Stop
            }
            _ => Propagation::Continue,
        })
        .id()
}
```

The events are the variants of `ElementEvent`: `Pointer`, `Key`, `Accessibility`, `TextChanged`,
`Drag` and `DragEnded`. For a press, a component of the kit is nearly always the better choice:
`Button` already answers the click, Space, Return and an assistive technology.

Other handlers of the builder:

| Method | What it registers |
|---|---|
| `shortcut(chord, handler)` | A `KeyChord` that answers while focus is on the element or inside it; on the root of a window, also while nothing has focus. |
| `menu_command(menu, title, chord, handler)` | A shortcut that is shown in the menu bar as well, on the line `title` of the menu `menu`. |
| `drag_source(start)`, `drop_target(types)` | A drag that begins on the element, and the drags the element takes. `crates/catalog/src/pages/images_page.rs` shows both. |
| `held_for(when, delay)` | No handler: a signal that tells whether `when` has held for `delay` without a break — what a tooltip waits for. |

A handler never touches a window: it writes signals, and it hands what opens or closes a window
to `Application::defer` ([An application](application.md#defer-what-a-handler-cannot-do-at-once)).
