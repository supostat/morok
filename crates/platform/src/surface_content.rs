use std::cell::RefCell;
use std::ops::Range;
use std::rc::Rc;
use std::time::Duration;

use morok_core::accesskit::{ActionRequest, TreeUpdate};
use morok_core::{
    ElementTree, FrameScheduler, KeyEvent, NativeMenu, NativeMenuItem, OverlaySide, Point, PointerInput, Rect,
    ScrollInput, Size, TextCommand,
};
use morok_style::ColorScheme;

use crate::focused_text::FocusedText;
use crate::overlay::overlay_id::OverlayId;
use crate::overlay::overlay_request::OverlayRequest;
use crate::surface_settings::SurfaceSettings;

/// What a window shows. The window asks its content for a frame whenever the tree of the
/// content, or the window itself, asked the frame loop for one.
pub trait SurfaceContent {
    /// Runs the stages of a frame for a surface of `viewport` shown with `scale` device pixels
    /// to a CSS pixel at `now`, the time the frame reaches the screen on the clock of the
    /// display, and hands the tree they ran on to `show`, once. The window draws the display
    /// list of the tree when the frame damaged anything.
    fn frame(&mut self, viewport: Size, scale: f32, now: Duration, show: &mut dyn FnMut(&ElementTree));

    /// The system or the application switched the appearance the surface is shown in.
    fn color_scheme_changed(&mut self, color_scheme: ColorScheme);

    /// The user turned "Reduce motion" on or off in the accessibility settings of the system.
    fn reduced_motion_changed(&mut self, reduces_motion: bool);

    /// The pointer moved over the surface, pressed or released a button on it, or left it.
    fn pointer_input(&mut self, input: &PointerInput);

    /// The wheel or the trackpad scrolled over the surface.
    fn scroll_input(&mut self, input: &ScrollInput);

    /// A key went down or up while the surface had the keyboard. Returns whether the content
    /// used the key; a key it did not use goes on to the window and the application.
    fn key_input(&mut self, event: &KeyEvent) -> bool;

    /// The whole accessibility tree of the surface: asked for once, when an assistive
    /// technology starts to read the surface.
    fn accessibility_tree(&mut self) -> TreeUpdate;

    /// What the last frame changed of the accessibility tree: asked for after every frame
    /// for as long as an assistive technology reads the surface.
    fn accessibility_update(&mut self) -> TreeUpdate;

    /// An assistive technology asks a node of the accessibility tree for an action.
    fn accessibility_action(&mut self, request: &ActionRequest);

    /// The text the focused element edits; none while focus is on no edited text. A content
    /// that edits no text keeps this.
    fn focused_text(&mut self) -> Option<FocusedText> {
        None
    }

    /// Inserts what was typed, dictated or settled on by an input method into the focused
    /// text: in place of the text being composed, else of `replacement`, else of the
    /// selection. Places are bytes of the text.
    fn insert_text(&mut self, _text: &str, _replacement: Option<Range<usize>>) {}

    /// Shows what an input method composes in the focused text, with the bytes `selected` of
    /// it selected; an empty text gives the composition up.
    fn set_marked_text(&mut self, _text: &str, _selected: Range<usize>, _replacement: Option<Range<usize>>) {}

    /// Settles the text an input method composes as it stands.
    fn unmark_text(&mut self) {}

    /// Does a command of the Edit menu to the focused text. Returns whether a text took it.
    fn text_command(&mut self, _command: TextCommand) -> bool {
        false
    }

    /// The box of the first line of a range of the focused text, in CSS pixels of the surface.
    fn text_range_box(&mut self, _range: Range<usize>) -> Option<Rect> {
        None
    }

    /// The place of the focused text nearest to a point of the surface.
    fn text_offset_at(&mut self, _point: Point) -> Option<usize> {
        None
    }

    /// Lists the overlays the content wants shown over the surface now into `overlays`: asked
    /// after every frame of the surface, at `now`. An overlay is laid out in `available`, the
    /// room the screen of the surface has for it, with `scale` device pixels to a CSS pixel.
    /// The window opens the overlays that are new, places every one, and closes the ones
    /// that are listed no longer. A content that opens no overlay keeps this.
    fn overlays(&mut self, _available: Size, _scale: f32, _now: Duration, _overlays: &mut Vec<OverlayRequest>) {}

    /// The content of an overlay the content listed: asked once, when the overlay is first
    /// shown. It is given the frame scheduler of the window of the overlay and the settings
    /// of the system that window opens under.
    ///
    /// # Panics
    ///
    /// The default does: a content that lists an overlay gives its content.
    fn overlay_content(
        &mut self,
        overlay: OverlayId,
        _frame_scheduler: Box<dyn FrameScheduler>,
        _settings: SurfaceSettings,
    ) -> Rc<RefCell<dyn SurfaceContent>> {
        panic!("the content listed {overlay:?} and gives no content for it");
    }

    /// The window placed an overlay: on `side` of its anchor, with the top left corner of its
    /// surface at `origin` in CSS pixels of this surface. Told when either changes.
    fn overlay_placed(&mut self, _overlay: OverlayId, _side: OverlaySide, _origin: Point) {}

    /// The user turned away from the surface and its overlays: went to another application,
    /// or to a window that shows no surface.
    fn dismiss_overlays(&mut self) {}

    /// The menu of the system the content asks for, once: the window shows it and tells the
    /// command the user chose through
    /// [`native_menu_command_chosen`](Self::native_menu_command_chosen). Asked after every
    /// frame of the surface.
    fn native_menu(&mut self) -> Option<NativeMenu> {
        None
    }

    /// The user chose the command `id` of the menu of the system the window showed last.
    fn native_menu_command_chosen(&mut self, _id: usize) {}

    /// How many times the menus the content gives the menu bar changed: the window builds
    /// the bar anew when the number is another than the one it saw last. Asked after every
    /// frame of the surface of a window.
    fn menu_bar_revision(&mut self) -> u64 {
        0
    }

    /// The menus the content gives the menu bar, each a submenu of commands: the window shows
    /// them, after the menu of the application, while it has the keyboard, and tells the
    /// command the user chose through
    /// [`menu_bar_command_chosen`](Self::menu_bar_command_chosen). A content with no commands
    /// keeps this.
    fn menu_bar(&mut self) -> Vec<NativeMenuItem> {
        Vec::new()
    }

    /// The user chose the command `id` of the menus the content gave the menu bar.
    fn menu_bar_command_chosen(&mut self, _id: usize) {}
}
