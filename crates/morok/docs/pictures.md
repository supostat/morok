# Pictures

Three kinds of element show something that is no box and no text: an image the application
loaded, an icon of the kit, and a canvas the application draws itself.

## Images

An application hands the kit the **encoded bytes** of a picture — PNG, JPEG, HEIC or any format
the system reads — and gets an `ImageId`. The kit decodes off the main thread. The bytes are the
one way in: the application reads its files itself.

```rust,no_run
use morok::components::Image;
use morok::core::Size;
use morok::{Application, RendererError, Styles, WindowOptions};

fn main() -> Result<(), RendererError> {
    let application = Application::new(Styles::of_kit())?;
    let encoded = std::fs::read("meadow.png").expect("the picture is readable");
    let meadow = application.add_image(encoded);

    let options = WindowOptions {
        title: String::from("Meadow"),
        content_size: Size {
            width: 640.0,
            height: 480.0,
        },
        blur_behind: false,
    };
    let _window = application.open_window(&options, move |mount| {
        Image::new(meadow, "A meadow in the morning").mount(mount)
    });
    application.run();
    Ok(())
}
```

- `Image::new(image, label)`: `label` is what an assistive technology reads. The image is a
  property: a `Signal<ImageId>` shows whichever picture it holds.
- An image is shown at its natural size unless its style gives it a width or a height; then
  `object-fit` says how the picture fills the box.
- While the picture decodes, and when its bytes are no picture, the element is its box alone.
  `Application::image_status` tells which: `ImageStatus::Decoding`, `Unreadable` or
  `Decoded(size)`.
- An `ImageId` is `Copy`: one picture may be shown by any number of elements, in any window of
  the application.

The catalog shows a PNG and a JPEG under every `object-fit`, and a drop zone that takes an image
file from Finder: `crates/catalog/src/pages/images_page.rs`, with the pictures added in
`crates/catalog/src/gallery.rs`.

## Icons

The icons of the kit are the variants of `morok::components::KitIcon`; `KitIcon::ALL` lists them.
`Mount::kit_icon` creates an element that shows one.

```rust
use morok::components::KitIcon;
use morok::components::tokens::TOKENS;
use morok::components::view::Mount;
use morok::core::ElementId;
use morok::style::{ColorValue, Declaration};

fn folder_icon(mount: &mut Mount<'_>) -> ElementId {
    mount
        .kit_icon(KitIcon::Folder)
        .inline(Declaration::Color(ColorValue::Token(TOKENS.color_accent)))
        .id()
}
```

- An icon has no size of its own: its style gives it a width and a height. In the catalog that is
  the class `.icon-glyph` of `crates/catalog/styles/pages.css`.
- An icon drawn in `currentColor` takes the `color` of its element, so it follows the theme like
  a text.
- The components take an icon as a `KitIcon`: `IconButton::new(icon, label)`,
  `SidebarItem::icon`, `MenuItem::icon`.
- The set is closed: an application shows a picture of its own as an image or draws it on a
  canvas. The icons are the SVG files of `design/icons`, and the catalog shows them by name in
  `crates/catalog/src/pages/icons_page.rs`.

## Canvases

A canvas is an element the kit lays out, clips and orders like any other, and whose content the
application draws through the renderer of the kit. Two things make one:

1. An element created with `Mount::canvas(canvas)`, where `canvas` is a `CanvasId`: a number the
   application chose for it.
2. A painter, a type that implements `CanvasPainter`, named once for the whole application with
   `Application::set_canvas_painter`. The kit calls `draw` once per canvas per frame, in paint
   order, with the pass clipped to the canvas.

```rust,no_run
use morok::components::tokens::TOKENS;
use morok::core::{CanvasId, Rect, Size};
use morok::style::{Declaration, LengthPercentage, LengthPercentageAuto, Pixels};
use morok::{Application, CanvasEncoder, CanvasPainter, RendererError, Styles, WindowOptions};

const LEVEL_METER: CanvasId = CanvasId(1);

/// Draws a bar filled to three quarters, in the accent color of the theme.
struct MeterPainter;

impl CanvasPainter for MeterPainter {
    fn draw(&mut self, canvas: CanvasId, encoder: &mut CanvasEncoder<'_>) {
        if canvas != LEVEL_METER {
            return;
        }
        let region = encoder.region();
        let accent = *encoder.theme().resolve(TOKENS.color_accent);
        encoder.fill(Rect::new(0.0, 0.0, region.width * 0.75, region.height), accent);
    }
}

fn pixels(pixels: f32) -> LengthPercentageAuto {
    LengthPercentageAuto::LengthPercentage(LengthPercentage {
        pixels: Pixels(pixels),
        fraction: None,
        tokens: std::borrow::Cow::Borrowed(&[]),
    })
}

fn main() -> Result<(), RendererError> {
    let application = Application::new(Styles::of_kit())?;
    application.set_canvas_painter(MeterPainter);
    let options = WindowOptions {
        title: String::from("Level"),
        content_size: Size {
            width: 320.0,
            height: 80.0,
        },
        blur_behind: false,
    };
    let _window = application.open_window(&options, |mount| {
        mount
            .canvas(LEVEL_METER)
            .inline(Declaration::Width(pixels(280.0)))
            .inline(Declaration::Height(pixels(16.0)))
            .id()
    });
    application.run();
    Ok(())
}
```

- A canvas has no size of its own: its style gives it a width and a height.
- What `draw` is given is in CSS pixels from the top left corner of the canvas:
  `CanvasEncoder::region` is its content box, with the scale of the device pixels.
- `fill` fills a rectangle and `fill_ring_sector` a `RingSector`, the part of a ring between two
  angles. `metal_encoder` is the Metal encoder of the pass, for an application that draws with
  pipelines of its own.
- `CanvasEncoder::theme` holds the values of the tokens of the window: a painter that resolves
  its colors there follows the theme and the color scheme without being told of a change.
- The painter draws what it holds. Data that changes reaches it through what the application
  shares with it — an `Rc<RefCell<…>>`, or one canvas per state, chosen by a signal: the canvas
  is a property, and a `Signal<CanvasId>` or a `Memo<CanvasId>` shows whichever it holds.

The disk map of the catalog draws a sunburst and a treemap this way: the element in
`crates/catalog/src/disk_map.rs`, the painter in `crates/catalog/src/disk_map_painter.rs`.
