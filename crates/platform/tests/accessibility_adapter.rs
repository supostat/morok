mod support;

use std::process::ExitCode;
use std::time::Duration;

use morok_core::accesskit::{ActionRequest, Role as NodeRole, TreeUpdate};
use morok_core::{
    AccessibilityAction, AccessibleActions, ElementId, ElementTree, Focusability, InputHandler, KeyEvent, PointerEvent,
    PointerInput, Propagation, Rect, ShortcutId, Size,
};
use morok_platform::{Application, SurfaceContent, Window, WindowOptions};
use morok_style::ColorScheme;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{MainThreadMarker, msg_send};
use objc2_app_kit::{NSApplication, NSView, NSWindow, NSWindowDidBecomeKeyNotification};
use objc2_foundation::{NSArray, NSNotificationCenter, NSPoint, NSRect, NSSize, NSString};
use support::{Expectations, WindowScene, run_main_loop};

const PANEL: &str = ".panel { display: flex; gap: 20px; padding: 20px; background-color: var(--color-paper); }\n\
    .button { width: 100px; height: 30px; background-color: var(--color-blue); }";

const CONTENT_SIZE: Size = Size {
    width: 400.0,
    height: 300.0,
};
/// Long enough for a window to come on screen and for its first frames to be drawn.
const SETTLING: Duration = Duration::from_millis(1500);
/// Long enough for a requested frame to be drawn.
const ONE_FRAME: Duration = Duration::from_millis(500);
/// How far a frame the system reports may lie from the box it stands for, in points.
const FRAME_TOLERANCE: f64 = 0.01;

/// What assistive technologies asked of the elements of the tree.
#[derive(Debug, Default)]
struct Asked {
    actions: Vec<(ElementId, String)>,
}

impl InputHandler for Asked {
    fn pointer_event(&mut self, _element: ElementId, _event: &PointerEvent) -> Propagation {
        Propagation::Continue
    }

    fn key_event(&mut self, _element: ElementId, _event: &KeyEvent) -> Propagation {
        Propagation::Continue
    }

    fn shortcut(&mut self, _shortcut: ShortcutId) {}

    fn accessibility_action(&mut self, element: ElementId, action: &AccessibilityAction<'_>) {
        self.actions.push((element, format!("{action:?}")));
    }

    fn text_changed(&mut self, _element: ElementId, _text: &str) {}
}

/// A scene in a window that answers for the accessibility of its tree.
struct AccessibleScene {
    window_scene: WindowScene,
    asked: Asked,
}

impl SurfaceContent for AccessibleScene {
    fn frame(&mut self, viewport: Size, scale: f32, now: Duration, show: &mut dyn FnMut(&ElementTree)) {
        self.window_scene.frame(viewport, scale, now, show);
    }

    fn color_scheme_changed(&mut self, color_scheme: ColorScheme) {
        self.window_scene.color_scheme_changed(color_scheme);
    }

    fn reduced_motion_changed(&mut self, reduces_motion: bool) {
        self.window_scene.reduced_motion_changed(reduces_motion);
    }

    fn pointer_input(&mut self, _input: &PointerInput) {}

    fn key_input(&mut self, _event: &KeyEvent) -> bool {
        false
    }

    fn accessibility_tree(&mut self) -> TreeUpdate {
        self.window_scene.accessibility_tree()
    }

    fn accessibility_update(&mut self) -> TreeUpdate {
        self.window_scene.accessibility_update()
    }

    fn accessibility_action(&mut self, request: &ActionRequest) {
        self.window_scene
            .scene
            .tree
            .accessibility_action(request, &mut self.asked);
    }
}

/// An accessibility element of the system, asked as an assistive technology asks it.
struct AccessibleElement(Retained<AnyObject>);

impl AccessibleElement {
    fn role(&self) -> Option<String> {
        // SAFETY: an accessibility element answers `accessibilityRole` with a string or nil.
        text_of(unsafe { msg_send![&*self.0, accessibilityRole] })
    }

    fn title(&self) -> Option<String> {
        // SAFETY: an accessibility element answers `accessibilityTitle` with a string or nil.
        text_of(unsafe { msg_send![&*self.0, accessibilityTitle] })
    }

    /// The value of the element when it is a text.
    fn value(&self) -> Option<String> {
        // SAFETY: an accessibility element answers `accessibilityValue` with an object or nil.
        text_of(unsafe { msg_send![&*self.0, accessibilityValue] })
    }

    /// The frame of the element on the screen.
    fn frame(&self) -> NSRect {
        // SAFETY: an accessibility element answers `accessibilityFrame` with a rectangle.
        unsafe { msg_send![&*self.0, accessibilityFrame] }
    }

    fn children(&self) -> Vec<Self> {
        // SAFETY: an accessibility element answers `accessibilityChildren` with an array or nil.
        let children: Option<Retained<NSArray<AnyObject>>> = unsafe { msg_send![&*self.0, accessibilityChildren] };
        children.map_or_else(Vec::new, |children| children.to_vec().into_iter().map(Self).collect())
    }

    fn take_focus(&self) {
        // SAFETY: an accessibility element takes `setAccessibilityFocused:` with a flag.
        let (): () = unsafe { msg_send![&*self.0, setAccessibilityFocused: true] };
    }

    /// Returns whether the element performed its default action.
    fn press(&self) -> bool {
        // SAFETY: an accessibility element answers `accessibilityPerformPress` with a flag.
        unsafe { msg_send![&*self.0, accessibilityPerformPress] }
    }
}

/// The text an attribute holds; none for an attribute that is not set or holds no text.
fn text_of(attribute: Option<Retained<AnyObject>>) -> Option<String> {
    attribute
        .and_then(|value| value.downcast::<NSString>().ok())
        .map(|text| text.to_string())
}

fn is_near(frame: NSRect, expected: NSRect) -> bool {
    [
        frame.origin.x - expected.origin.x,
        frame.origin.y - expected.origin.y,
        frame.size.width - expected.size.width,
        frame.size.height - expected.size.height,
    ]
    .into_iter()
    .all(|difference| difference.abs() <= FRAME_TOLERANCE)
}

/// The window of the test with what the expectations read of it.
struct Stage {
    window: Window<AccessibleScene>,
    native_window: Retained<NSWindow>,
    surface_view: Retained<NSView>,
    expectations: Expectations,
    save: ElementId,
    cancel: ElementId,
}

impl Stage {
    /// A window on a panel with a line of text and two buttons that take focus and a click.
    fn open(application: &Application, main_thread: MainThreadMarker) -> Self {
        let options = WindowOptions {
            title: String::from("morok: accessibility adapter"),
            content_size: CONTENT_SIZE,
        };
        let mut buttons = None;
        let window = Window::open(application, &options, |frame_scheduler, settings| {
            let mut window_scene = WindowScene::new(PANEL, frame_scheduler, settings);
            let scene = &mut window_scene.scene;
            let root = scene.root();
            scene.add_classes(root, &["panel"]);
            let notice = scene.add_text(root, "Unsaved changes");
            scene.tree.set_accessible_role(notice, Some(NodeRole::Label));
            let save = scene.add(root, &["button"]);
            let cancel = scene.add(root, &["button"]);
            for (button, name) in [(save, "Save"), (cancel, "Cancel")] {
                scene.tree.set_accessible_role(button, Some(NodeRole::Button));
                scene.tree.set_accessible_name(button, name);
                scene.tree.set_accessible_actions(button, AccessibleActions::CLICK);
                scene.tree.set_focusability(button, Focusability::TabStop);
            }
            buttons = Some((save, cancel));
            AccessibleScene {
                window_scene,
                asked: Asked::default(),
            }
        });
        let (save, cancel) = buttons.expect("the window built its content");
        let native_window = NSApplication::sharedApplication(main_thread)
            .windows()
            .firstObject()
            .expect("the application has opened its window");
        let surface_view = native_window.contentView().expect("the window shows its surface");
        Self {
            window,
            native_window,
            surface_view,
            expectations: Expectations::default(),
            save,
            cancel,
        }
    }

    /// The elements the view shows to the system, the one of the root first and alone.
    fn elements_of_the_view(&self) -> Vec<AccessibleElement> {
        AccessibleElement(Retained::into_super(Retained::into_super(Retained::into_super(
            self.surface_view.clone(),
        ))))
        .children()
    }

    /// The elements of the text and of the two buttons, in tree order.
    fn elements_of_the_panel(&self) -> Vec<AccessibleElement> {
        self.elements_of_the_view()
            .first()
            .map_or_else(Vec::new, AccessibleElement::children)
    }

    /// The rectangle of the screen that shows a box of the surface.
    fn on_screen(&self, border_box: Rect) -> NSRect {
        let in_view = NSRect::new(
            NSPoint::new(f64::from(border_box.x), f64::from(border_box.y)),
            NSSize::new(f64::from(border_box.width), f64::from(border_box.height)),
        );
        self.native_window
            .convertRectToScreen(self.surface_view.convertRect_toView(in_view, None))
    }

    fn border_box(&self, element: ElementId) -> Rect {
        self.window
            .update(|content| content.window_scene.scene.border_box(element))
    }

    fn the_first_frame_lays_the_tree_out(&mut self) {
        run_main_loop(SETTLING);
        let statistics = self.window.frame_statistics();
        self.expectations.expect(
            statistics.presented_frames >= 1,
            "the window lays its tree out in a first frame; is the display on?",
            &format!("{statistics:?}"),
        );
    }

    fn the_view_shows_the_nodes_of_the_tree(&mut self) {
        let of_the_view: Vec<Option<String>> = self
            .elements_of_the_view()
            .iter()
            .map(AccessibleElement::role)
            .collect();
        self.expectations.expect(
            of_the_view == [Some(String::from("AXGroup"))],
            "the view has one accessibility child, the group of the root",
            &format!("{of_the_view:?}"),
        );
        let panel = self.elements_of_the_panel();
        let roles: Vec<Option<String>> = panel.iter().map(AccessibleElement::role).collect();
        let expected_roles = ["AXStaticText", "AXButton", "AXButton"].map(|role| Some(String::from(role)));
        self.expectations.expect(
            roles == expected_roles,
            "the root holds the text and the two buttons, in tree order",
            &format!("{roles:?}"),
        );
        let named: Vec<Option<String>> = panel
            .iter()
            .map(|element| element.title().or_else(|| element.value()))
            .collect();
        let expected_names = ["Unsaved changes", "Save", "Cancel"].map(|name| Some(String::from(name)));
        self.expectations.expect(
            named == expected_names,
            "the text gives its text as its value and the buttons their names as titles",
            &format!("{named:?}"),
        );
    }

    fn an_element_lies_where_its_box_is_shown(&mut self) {
        let panel = self.elements_of_the_panel();
        let Some(save) = panel.get(1) else {
            self.expectations
                .expect(false, "the first button has an element", "none");
            return;
        };
        let (frame, expected) = (save.frame(), self.on_screen(self.border_box(self.save)));
        self.expectations.expect(
            is_near(frame, expected),
            "the frame of an element is the box of its node on the screen",
            &format!("{frame:?}, the box is at {expected:?}"),
        );

        let over_cancel = self.on_screen(self.border_box(self.cancel));
        let point = NSPoint::new(
            over_cancel.origin.x + over_cancel.size.width / 2.0,
            over_cancel.origin.y + over_cancel.size.height / 2.0,
        );
        // SAFETY: a view answers `accessibilityHitTest:` for a point of the screen with an
        // element or nil.
        let hit: Option<Retained<AnyObject>> = unsafe { msg_send![&*self.surface_view, accessibilityHitTest: point] };
        let hit_title = hit.map(AccessibleElement).and_then(|element| element.title());
        self.expectations.expect(
            hit_title.as_deref() == Some("Cancel"),
            "a point of the screen over the second button hits its element",
            &format!("{hit_title:?}"),
        );
    }

    fn a_focus_action_moves_the_focus_of_the_tree(&mut self) {
        let panel = self.elements_of_the_panel();
        if let Some(save) = panel.get(1) {
            save.take_focus();
        }
        let focused = self.window.update(|content| content.window_scene.scene.tree.focused());
        self.expectations.expect(
            focused == Some(self.save),
            "asking the element of the first button to take focus focuses the button in the tree",
            &format!("{focused:?}"),
        );

        run_main_loop(ONE_FRAME);
        // SAFETY: the name is a constant of `AppKit` and the object is the window it is about.
        unsafe {
            NSNotificationCenter::defaultCenter()
                .postNotificationName_object(NSWindowDidBecomeKeyNotification, Some(&self.native_window));
        }
        // SAFETY: a view answers `accessibilityFocusedUIElement` with an element or nil.
        let focused_element: Option<Retained<AnyObject>> =
            unsafe { msg_send![&*self.surface_view, accessibilityFocusedUIElement] };
        let focused_title = focused_element
            .map(AccessibleElement)
            .and_then(|element| element.title());
        self.expectations.expect(
            focused_title.as_deref() == Some("Save"),
            "once its window has the keyboard, the view names the element of the focused button",
            &format!("{focused_title:?}"),
        );
    }

    fn a_press_is_told_to_the_element(&mut self) {
        let panel = self.elements_of_the_panel();
        let pressed = panel.get(2).is_some_and(AccessibleElement::press);
        let asked = self.window.update(|content| content.asked.actions.clone());
        self.expectations.expect(
            pressed && asked == [(self.cancel, String::from("Click"))],
            "pressing the element of the second button tells the button of a click",
            &format!("pressed: {pressed}, asked: {asked:?}"),
        );
    }

    fn a_frame_tells_the_adapter_what_changed(&mut self) {
        let save = self.save;
        self.window.update(|content| {
            content.window_scene.scene.tree.set_accessible_name(save, "Save all");
        });
        run_main_loop(ONE_FRAME);
        let titles: Vec<Option<String>> = self
            .elements_of_the_panel()
            .iter()
            .map(AccessibleElement::title)
            .collect();
        self.expectations.expect(
            titles == [None, Some(String::from("Save all")), Some(String::from("Cancel"))],
            "a name changed in the tree is the title of the element after the next frame",
            &format!("{titles:?}"),
        );
    }
}

fn main() -> ExitCode {
    let main_thread = MainThreadMarker::new().expect("the test runs on the main thread");
    let application = Application::new().expect("this Mac runs Metal 4");
    let mut stage = Stage::open(&application, main_thread);

    stage.the_first_frame_lays_the_tree_out();
    stage.the_view_shows_the_nodes_of_the_tree();
    stage.an_element_lies_where_its_box_is_shown();
    stage.a_focus_action_moves_the_focus_of_the_tree();
    stage.a_press_is_told_to_the_element();
    stage.a_frame_tells_the_adapter_what_changed();

    stage.expectations.report("accessibility adapter")
}
