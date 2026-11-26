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
| `accessible_role(role)`, `accessible_name(name)` and the other `accessible_…` methods | Its node in the accessibility tree ([below](#accessibility)). The role comes first: the others panic on an element without one. |

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
`Declaration` is one longhand property with its value. The enum is `morok::style::Declaration`,
and `crates/style/src/declaration.rs` lists its variants: that file is where to look a property
up.

- A variant is named after its longhand: `flex-grow` is `Declaration::FlexGrow`, `column-gap` is
  `Declaration::ColumnGap`. A shorthand has none: `gap` is a `RowGap` and a `ColumnGap`.
- The longhands of the four sides share one variant, which takes the side first: `padding-top` is
  `Declaration::Padding(Side::Top, …)`, and `Margin`, `Inset`, `BorderWidth`, `BorderStyle` and
  `BorderColor` are written the same way. The sides are `Side::Top`, `Side::Right`,
  `Side::Bottom` and `Side::Left`, and `Side::ALL` is the four of them. `BorderRadius` takes a
  `Corner` in the same way.
- A keyword is a variant of its enum, written in camel case: `center` is
  `ItemAlignment::Center`, `flex-start` is `ItemAlignment::FlexStart`, `space-between` is
  `ContentAlignment::SpaceBetween`, `nowrap` is `FlexWrap::NoWrap`. `morok::style` exports the
  enums; each is declared in a file of its own under `crates/style/src/layout`,
  `crates/style/src/paint`, `crates/style/src/text` or `crates/style/src/effect`, and the
  imports at the top of `crates/style/src/declaration.rs` name the file of each.
- A number is an `f32`: `Declaration::FlexGrow(1.0)`, `Declaration::Opacity(0.5)`.
- A color is a `ColorValue`: a token of `morok::components::tokens::TOKENS`,
  `ColorValue::Transparent` or `ColorValue::CurrentColor`.
- A length is written in full, as shown below.

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

A length is written in full: pixels, a fraction of the percentage basis and token terms. Two
functions of the application keep that in one place; the examples of this chapter call them
`pixels` and `space`.

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

/// The length a token holds under the theme of the window.
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

A declaration reaches an element a component built through `Mount::build`: this is how a field
is told to take the room its row leaves.

```rust
use morok::components::view::Mount;
use morok::core::ElementId;
use morok::style::{Declaration, Display};

fn search_row(mount: &mut Mount<'_>, field: ElementId, button: ElementId) -> ElementId {
    mount.build(field).inline(Declaration::FlexGrow(1.0));
    mount
        .element()
        .inline(Declaration::Display(Display::Flex))
        .child(field)
        .child(button)
        .id()
}
```

What an element looks like before any declaration reaches it:

- `display: block`, no margin, no padding, `box-sizing: border-box`. Every other property of
  layout has its initial value of CSS: a flex container is a row that does not wrap, and nothing
  in it grows.
- The children of a block stand one under another, each as wide as the block. A text is such a
  child, and breaks into lines at that width. There is no inline flow: two texts in one element
  are two boxes, one under the other, and what stands side by side stands in a flex container.
- As tall as its content, the outermost element of a window included.
- A transparent background, the root of the window included.
- Text in the font of the system at 16 pixels, in black. The color and the properties of text
  pass from an element to its children.

## The outermost element of a window

An application on the styles of the kit alone gives its outermost element what a stylesheet would
give its window: the height of the surface, a background, the color and the font of its texts,
and room around its content.

- **The height.** Without `height: 100%` the element ends where its content does, and so does its
  background.
- **The font.** The components set their own texts in `--font-ui` at `--text-md`, 13 pixels. A
  text the application creates with `Mount::text` takes the font and the color of the elements
  around it: given here once, they hold for all of them.
- **The background.** The catalog paints its window with `--color-surface`.

```rust
use std::borrow::Cow;

use morok::components::tokens::TOKENS;
use morok::components::view::Mount;
use morok::core::ElementId;
use morok::style::{
    ColorValue, Declaration, Display, FlexDirection, Length, LengthPercentage, LengthPercentageAuto, LineHeight,
    Pixels, Side, Token, TokenTerm,
};

/// The length a token holds under the theme of the window.
fn space(token: Token<Pixels>) -> LengthPercentage {
    LengthPercentage {
        pixels: Pixels(0.0),
        fraction: None,
        tokens: Cow::Owned(vec![TokenTerm { token, factor: 1.0 }]),
    }
}

/// The outermost element of a window: a column over the whole surface, in the colors and the
/// font of the kit, with room around what it holds.
fn window(mount: &mut Mount<'_>, content: &[ElementId]) -> ElementId {
    let whole_surface = LengthPercentageAuto::LengthPercentage(LengthPercentage {
        pixels: Pixels(0.0),
        fraction: Some(1.0),
        tokens: Cow::Borrowed(&[]),
    });
    let room = space(TOKENS.space_4);
    let mut window = mount.element();
    window
        .inline(Declaration::Height(whole_surface))
        .inline(Declaration::Display(Display::Flex))
        .inline(Declaration::FlexDirection(FlexDirection::Column))
        .inline(Declaration::RowGap(room.clone()))
        .inline(Declaration::BackgroundColor(ColorValue::Token(TOKENS.color_surface)))
        .inline(Declaration::Color(ColorValue::Token(TOKENS.color_text)))
        .inline(Declaration::FontFamily(TOKENS.font_ui))
        .inline(Declaration::FontSize(Length {
            pixels: Pixels(0.0),
            tokens: Cow::Owned(vec![TokenTerm {
                token: TOKENS.text_md,
                factor: 1.0,
            }]),
        }))
        .inline(Declaration::LineHeight(LineHeight::Token(TOKENS.text_leading)));
    for side in Side::ALL {
        window.inline(Declaration::Padding(side, room.clone()));
    }
    for child in content {
        window.child(*child);
    }
    window.id()
}
```

The component of a window calls it last, with what it built: `window(mount, &[search, notes])`.

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

## Accessibility

An element is in the accessibility tree only when it has a role: an element without one is
passed over, and its children are read as the children of the element around it. The components
of the kit give their elements roles, names and actions themselves. What an application builds
of its own elements gets none until the application gives it one.

- The role is a `morok::core::accesskit::Role`.
- A text with a role gives its text to its node. A text without one is not read at all.
- A box with a role is named by `accessible_name`, and says what may be asked of it with
  `accessible_actions`.

```rust
use morok::components::view::Mount;
use morok::core::ElementId;
use morok::core::accesskit::Role;

fn notes(mount: &mut Mount<'_>, texts: &[&str]) -> ElementId {
    let items: Vec<ElementId> = texts
        .iter()
        .map(|text| mount.text(*text).accessible_role(Role::ListItem).id())
        .collect();
    let mut list = mount.element();
    list.accessible_role(Role::List).accessible_name("Notes");
    for item in items {
        list.child(item);
    }
    list.id()
}
```
