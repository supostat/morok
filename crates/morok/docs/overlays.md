# Overlays

Everything that floats over a window is shown by a component, and **open while a signal of the
application holds**. The application opens an overlay by writing `true` to the signal; the overlay
writes `false` when it closes itself — on a press outside it, on Escape, on a command chosen.
Nothing is opened or closed by a call.

The catalog shows them all on one page, `crates/catalog/src/pages/overlays_page.rs`.

## Two kinds

| Kind | Components | Where it lives |
|---|---|---|
| A panel beside a control | `Popover`, `Menu`, `Tooltip`, the menu of a `ContextMenu`, the lists of `Select` and `Combobox` | In an overlay of its own: a window without a frame, which may reach past the window it belongs to. It follows its anchor when the window moves. |
| A layer over the window | `Dialog`, `Sheet`, `ToastStack` | In the tree of the window, as an element the application puts there. |

## Who owns an overlay

- A **panel** belongs to an element of the window: its anchor, or the box built around its target.
  When that element is removed, or the window closes, the panel goes with it. So the application
  holds nothing but the signal.
- A **layer** is an element like any other: it lives where the application put it, and covers
  the box it is positioned against — the nearest ancestor with a `position`, or the whole window
  when there is none.

## Popover and menu: attached to an anchor

`attach(mount, anchor)` shows the panel under `anchor`, an element of the window, while the
signal holds; it returns nothing, because it adds nothing to the tree. A press on the anchor is
left to the anchor: the anchor is what opens and closes the panel.

```rust
use morok::components::view::Mount;
use morok::components::{Button, Menu, MenuItem, Popover};
use morok::core::ElementId;
use morok::reactive::Signal;

/// A button that opens what `is_open` shows, and closes it again.
fn toggle(mount: &mut Mount<'_>, label: &str, is_open: Signal<bool>) -> ElementId {
    Button::new(label)
        .on_press(move |runtime| runtime.update(is_open, |is_open| *is_open = !*is_open))
        .mount(mount)
}

fn document_actions(mount: &mut Mount<'_>) -> ElementId {
    let is_menu_open = mount.runtime().signal(false);
    let is_info_open = mount.runtime().signal(false);
    let chosen = mount.runtime().signal(String::new());

    let more = toggle(mount, "More", is_menu_open);
    Menu::new(
        is_menu_open,
        vec![
            MenuItem::new("Duplicate").on_choose(move |runtime| runtime.set(chosen, String::from("duplicate"))),
            MenuItem::submenu("Export", vec![MenuItem::new("PDF"), MenuItem::new("Plain text")]),
            MenuItem::separator(),
            MenuItem::new("Move to Trash")
                .destructive()
                .on_choose(move |runtime| runtime.set(chosen, String::from("trash"))),
        ],
    )
    .label("Document actions")
    .attach(mount, more);

    let info = toggle(mount, "Info", is_info_open);
    Popover::new(is_info_open)
        .title("Notes.txt")
        .message("12 KB, changed today.")
        .content(move |mount| {
            Button::new("Done")
                .on_press(move |runtime| runtime.set(is_info_open, false))
                .mount(mount)
        })
        .label("Document info")
        .attach(mount, info);

    mount.element().child(more).child(info).id()
}
```

- A `Menu` closes itself when a command is chosen. A `Popover` stays until a press outside it or
  Escape, or until the application writes `false`: a button inside it writes the signal, as
  above.
- The `content` of a popover is built anew every time it opens, and removed when it closes: the
  signals it creates do not outlive it.
- `MenuItem::shortcut` only shows a chord beside the label. The chord itself is registered with
  `ElementBuilder::shortcut` or `menu_command` ([Elements](elements.md#handlers)).

## Tooltip and context menu: built around a target

`mount(mount, target)` builds a box around `target` and returns it: the box goes into the tree in
place of the target.

```rust
use morok::components::view::Mount;
use morok::components::{Button, ContextMenu, MenuItem, Tooltip};
use morok::core::ElementId;

fn recalculate(mount: &mut Mount<'_>) -> ElementId {
    let button = Button::new("Recalculate").mount(mount);
    let hinted = Tooltip::new("Recalculate the sizes of every folder").mount(mount, button);
    ContextMenu::new(vec![MenuItem::new("Recalculate now"), MenuItem::new("Recalculate every hour")])
        .label("Recalculation")
        .mount(mount, hinted)
}
```

A `ContextMenu` opens on a press of the secondary button, where the pointer is. It is the menu of
the system unless `custom()` asks the kit to draw it.

## Dialog, sheet and toasts: layers of the window

`mount(mount)` returns an element that holds the layer while it is open. Put under the outermost
element of the window, it covers the window.

```rust
use morok::components::view::Mount;
use morok::components::{Button, Dialog, Toast, ToastStack, ToastTone};
use morok::core::ElementId;

fn trash(mount: &mut Mount<'_>) -> ElementId {
    let is_asking = mount.runtime().signal(false);
    let toasts = mount.runtime().signal(Vec::new());

    let empty = Button::new("Empty Trash…")
        .on_press(move |runtime| runtime.set(is_asking, true))
        .mount(mount);
    let dialog = Dialog::new(is_asking, "Empty the Trash?")
        .message("This cannot be undone.")
        .cancel_action("Cancel", |_| {})
        .default_action("Empty", move |runtime| {
            runtime.update(toasts, |toasts| {
                toasts.push(Toast {
                    id: 1,
                    title: String::from("Trash emptied"),
                    detail: None,
                    tone: ToastTone::Success,
                });
            });
        })
        .mount(mount);
    let toast_stack = ToastStack::new(toasts).mount(mount);
    mount.element().child(empty).child(dialog).child(toast_stack).id()
}
```

- A `Dialog` and a `Sheet` close themselves after either answer: the handler of the answer runs,
  then the signal is `false`. Return gives the default answer and Escape the cancelling one. While
  one is open, focus stays inside it, and goes back to where it was when it closes.
- A `Sheet` is built like a `Dialog` and hangs from the top edge.
- A `Toast` is a value the application pushes onto a `Signal<Vec<Toast>>`; the stack takes it off
  again when its `lifetime` has passed. The `id` tells the toasts of a stack apart: every toast
  gets a number of its own.
- In a window whose content scrolls, the layers are siblings of the scroll view, not children of
  what scrolls: `.page-layers` in `crates/catalog/styles/pages.css`.

## What the application must provide

The panels are styled by the classes of the kit. They are found by their IDs, so the style sheet
of the application must have been compiled with the stylesheets of the kit first: `Styles::of_kit`
and `generate_application` both do it, and opening a panel under any other style sheet panics.

`Window::frame_image` draws the window alone: a panel is a window of its own and is not in it.
