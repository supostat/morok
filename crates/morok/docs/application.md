# An application

## From an empty crate to a window

The manifest names one dependency, the facade. Under a git dependency cargo checks the whole
repository of the kit out, so the guide and the catalog travel with it.

```toml
[dependencies]
morok = { git = "https://github.com/supostat/morok" }
```

The smallest application creates an `Application` with the style tables of the kit, opens a
window on a component and runs the event loop:

```rust,no_run
use morok::components::Button;
use morok::core::Size;
use morok::{Application, RendererError, Styles, WindowOptions};

fn main() -> Result<(), RendererError> {
    let application = Application::new(Styles::of_kit())?;
    let options = WindowOptions {
        title: String::from("Hello"),
        content_size: Size {
            width: 480.0,
            height: 320.0,
        },
        blur_behind: false,
    };
    let _window = application.open_window(&options, |mount| Button::new("Press me").mount(mount));
    application.run();
    Ok(())
}
```

- `Application::new` is called on the main thread; it fails with a `RendererError` when Metal
  cannot give the renderer what it needs.
- `open_window` calls the component once and makes the element it returns the child of the root
  of the tree of the window. It returns a `Window`: **the window is on screen until that value is
  dropped**, so the application keeps it — here in a binding that lives as long as `main`.
- `content_size` is the size of the surface inside the frame of the window, in CSS pixels.
- `run` returns when the application quits.

The component is any `FnOnce(&mut Mount<'_>) -> ElementId`: a closure, as above, or a function of
the application. [Elements](elements.md) tells what it does with the `Mount`.

## Styles of its own

`Styles::of_kit()` holds the stylesheets of the components under the base theme of the kit, and
nothing else: an application built on it lays its own elements out with inline declarations
([Elements](elements.md#inline-declarations)) and gives its window a background and a font through
its outermost element ([Elements](elements.md#the-outermost-element-of-a-window)). An application with classes and tokens of its own
compiles them at build time. The catalog is the example to copy; its parts are:

| Part | In the catalog | What it does |
|---|---|---|
| The manifest | `crates/catalog/Cargo.toml` | `morok` as the dependency, `morok-components` and `morok-css` as build-dependencies. |
| The build script | `crates/catalog/build.rs` | Hands the tokens and the stylesheets of the kit, as `morok_components::KIT_TOKENS` and `morok_components::STYLESHEETS`, with the files of the application to `morok_css::build_script::generate_application`. |
| The tokens file | `crates/catalog/tokens/base.css` | Defines **every** token of the kit, under both color schemes, and may add tokens of its own. A token of the kit left out is the diagnostic `--x is not defined`. The canonical file to start from is `design/system/tokens.css`. |
| The stylesheets | `crates/catalog/styles` | Every `.css` file of the directory, compiled after the stylesheets of the kit. |
| Other values of the tokens | `crates/catalog/tokens/ember.css` | Optional: a file that gives exactly the tokens of the tokens file other values, for an application that switches themes. |
| The generated tables | `crates/catalog/src/styles.rs`, `crates/catalog/src/tokens.rs`, `crates/catalog/src/ember.rs` | Each includes one file of `OUT_DIR`: `styles.rs` declares `CLASSES` and `STYLE_SHEET`; `tokens.rs` declares `TOKENS`, `LIGHT` and `DARK`; `themes/ember.rs`, named after the file of the other values, declares `LIGHT` and `DARK`. |
| The styles of the application | `crates/catalog/src/catalog.rs` | The function `styles` builds `Styles { style_sheet: STYLE_SHEET, light: LIGHT, dark: DARK }` and hands it to `Application::new`. |
| The classes in use | `crates/catalog/src/page_layout.rs` | `CLASSES.page` is the class `.page` of `crates/catalog/styles/pages.css`: a class name in kebab case becomes a field in snake case. |

A build fails when the compiler refuses a stylesheet, with a diagnostic for every refusal that
names the file, the line and the column. [Styles](styles.md) tells what a stylesheet may hold.

The stylesheets of the kit are compiled first, in the order of the kit, so the classes of the kit
keep the IDs its components know them by. The overlays of the kit — popovers, menus, lists that
drop down — need that order, and `generate_application` keeps it.

## Themes and appearance

A window follows the appearance of the system: it shows the `light` table of the `Styles` under
the light color scheme and the `dark` one under the dark. Two calls change that while the
application runs:

- `Application::set_appearance` shows the application in `Appearance::Light` or
  `Appearance::Dark`, or lets it follow the system again with `Appearance::System`.
- `Application::set_themes` restyles every window with other values of the tokens: the `LIGHT`
  and `DARK` tables generated from a file of other values. The style sheet stays as it is.

```rust,no_run
use morok::{Appearance, Application, RendererError, Styles};

fn main() -> Result<(), RendererError> {
    let application = Application::new(Styles::of_kit())?;
    application.set_appearance(Appearance::Dark);
    let kit = Styles::of_kit();
    application.set_themes(kit.light, kit.dark);
    application.run();
    Ok(())
}
```

The catalog switches both from its toolbar: `crates/catalog/src/catalog.rs`, the function
`follow_settings`.

## Several windows

Every `open_window` gives a window with a tree of its own. The windows share one signal graph, so
one signal may be shown in several of them: a write to it changes them all.

- The application keeps every `Window`; dropping one closes it.
- The button of the frame closes a window too. `Window::on_closed` names what runs after that, and
  `Window::is_open` tells whether the window is still on screen. Everything mounted in a closed
  window is gone: its elements, and the signals and effects its components created.
- `Window::set_content_size`, `Window::viewport`, `Window::frame_image` and `Window::with_tree`
  reach the window from outside.

## `defer`: what a handler cannot do at once

A handler of a component runs in the middle of an event. There a window can be neither opened nor
dropped, and the themes cannot be switched: `open_window` and `set_themes` panic. A handler hands
such work to `Application::defer`, which runs it once the event is over. `Application` is a
handle: a clone of it goes into the handler.

```rust,no_run
use std::cell::RefCell;
use std::rc::Rc;

use morok::components::Button;
use morok::core::Size;
use morok::{Application, RendererError, Styles, Window, WindowOptions};

fn options(title: &str) -> WindowOptions {
    WindowOptions {
        title: title.to_owned(),
        content_size: Size {
            width: 360.0,
            height: 240.0,
        },
        blur_behind: false,
    }
}

fn main() -> Result<(), RendererError> {
    let application = Application::new(Styles::of_kit())?;
    let windows: Rc<RefCell<Vec<Window>>> = Rc::default();

    let opening = application.clone();
    let opened = Rc::clone(&windows);
    let first = application.open_window(&options("First"), move |mount| {
        Button::new("Open another window")
            .on_press(move |_| {
                let application = opening.clone();
                let opened = Rc::clone(&opened);
                opening.defer(move || {
                    let window = application.open_window(&options("Another"), |mount| {
                        Button::new("Nothing to do here").mount(mount)
                    });
                    opened.borrow_mut().push(window);
                });
            })
            .mount(mount)
    });
    windows.borrow_mut().push(first);
    application.run();
    Ok(())
}
```

The tree of a window holds its handlers, and this handler holds the windows: a cycle that keeps
them all until the process ends. That is right for an application that runs until it quits. One
that must let go of its windows earlier gives its handlers a `Weak` link instead, as the catalog
does with `CatalogLink` in `crates/catalog/src/catalog.rs`.

## Reaching the signals from outside

`Application::update` hands the signal graph to code that is no component: to create the signals
several windows share before any of them opens, or to write a signal from a timer or a callback
of the application. What a write changes shows in the next frame. [Signals](signals.md) has the
example.
