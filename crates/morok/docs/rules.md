# Rules

The kit does not guard against a mistake of the application with a quiet fallback: where a call
has no meaning, it panics, and the doc comment of the item says when under `# Panics`. This
chapter gathers what an application meets first.

## What a caller must keep

- **The main thread.** `Application::new` is called on it, and everything of the kit stays on it:
  `Application` and the signal graph are not `Send`. Work on another thread hands its result back
  to the main thread before it writes a signal.
- **The windows.** A `Window` closes when it is dropped. The application keeps every window it
  wants on screen.
- **Nothing that touches a window in the middle of an event.** A handler writes signals. It does
  not open or drop a window and does not switch themes: it hands that to `Application::defer`
  ([An application](application.md#defer-what-a-handler-cannot-do-at-once)).
- **No flush.** The kit runs the effects at the start of every frame; an application never calls
  `Runtime::flush`.
- **A handle only while its node lives.** A signal, memo or effect created in a component goes
  with what the component built ([Signals](signals.md#who-owns-a-node)).
- **The stylesheets of the kit first.** A style sheet of an application is compiled by
  `generate_application`, which puts the stylesheets of the kit before those of the application.
- **Every token of the kit.** The tokens file of an application defines them all.
- **A size for what scrolls.** A `List`, a `Tree`, a `Table` and a `ScrollView` scroll inside
  the box their style gives them, and a `SplitView` fills the box it is put in: the application
  gives each a height.
- **An accessible name.** A component that shows no text of its own is given a `label`.

## What panics

| Where | When |
|---|---|
| `Application::new` | Called from a thread other than the main one. |
| `Application::open_window`, `Application::set_themes` | Called while the application tells a handler of an event. |
| `Application::image_status` | The id names no picture the application added. |
| `Window::with_tree`, `Window::frame_image` | The button of the frame closed the window: its surface is gone. |
| A frame of a window | An effect keeps waking itself: it writes a signal it reads. |
| A use of a `Signal`, `Memo` or `Effect` | The scope that owned its node was disposed. |
| A read of a memo | The memo reads its own value while computing it. |
| `Runtime::flush` | Called from inside an effect. |
| `RadioGroup::mount`, `SegmentedControl::mount`, `Select::mount`, `Tabs::mount`, `Sidebar::mount` | The component has no options. |
| `SplitView::mount` | The view has not exactly one flexible pane. |
| `SplitPane::collapsible` | The pane is the flexible one. |
| `Slider::range` | The range holds one number or none, or the step does not move forward. |
| `MenuItem::on_choose` | The item is a submenu or a separator. |
| `ElementBuilder::placeholder` | The element edits no text. |
| `ElementBuilder::accessible_name` and the other `accessible_…` methods | The element has no accessible role yet. |
| Opening a popover, a menu or the list of a select | The style sheet was not compiled with the stylesheets of the kit first. |

## What is refused at build time

A stylesheet outside the subset, a token of the wrong kind, a token of the kit left out of the
tokens file: each fails the build with a diagnostic, never the running application
([Styles](styles.md#what-the-compiler-refuses-and-what-it-says)).

## What fails without a panic

- `Application::new` returns a `RendererError` when Metal cannot give the renderer what it needs.
- A picture whose bytes no decoder reads shows its box alone; `Application::image_status` answers
  `ImageStatus::Unreadable`.
- A handler that answers `Propagation::Continue` to every event uses none: the event goes on to
  the element around it.
