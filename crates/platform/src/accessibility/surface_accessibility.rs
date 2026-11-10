use std::cell::RefCell;
use std::ffi::c_void;
use std::fmt::{Debug, Formatter, Result};
use std::rc::Rc;

use accesskit_macos::Adapter;
use objc2::runtime::AnyObject;
use objc2_app_kit::NSView;
use objc2_foundation::NSPoint;

use crate::accessibility::content_actions::ContentActions;
use crate::accessibility::whole_tree::WholeTree;
use crate::surface_content::SurfaceContent;

/// The accessibility of one surface: the adapter that shows the accessibility tree of the
/// content to the assistive technologies of the system as elements of the view.
///
/// The adapter costs nothing until something asks the view about its elements. From then on
/// every frame tells it what changed.
pub(crate) struct SurfaceAccessibility {
    adapter: RefCell<Adapter>,
    content: Rc<RefCell<dyn SurfaceContent>>,
}

impl SurfaceAccessibility {
    /// `is_view_focused` says whether the window of the view has the keyboard now.
    pub(crate) fn new(view: &NSView, is_view_focused: bool, content: Rc<RefCell<dyn SurfaceContent>>) -> Self {
        let view_pointer: *const NSView = view;
        let actions = ContentActions {
            content: Rc::clone(&content),
        };
        // SAFETY: the pointer is the view, which is alive; the adapter keeps a weak reference
        // to it. Views are created on the main thread.
        let adapter = unsafe { Adapter::new(view_pointer.cast_mut().cast::<c_void>(), is_view_focused, actions) };
        Self {
            adapter: RefCell::new(adapter),
            content,
        }
    }

    /// The accessibility children of the view: an autoreleased array of the elements of the
    /// adapter.
    pub(crate) fn children(&self) -> *mut AnyObject {
        self.adapter
            .borrow_mut()
            .view_children(&mut self.whole_tree())
            .cast::<AnyObject>()
    }

    /// The element that has focus, or null when none has: an autoreleased element of the adapter.
    pub(crate) fn focused_element(&self) -> *mut AnyObject {
        self.adapter
            .borrow_mut()
            .focus(&mut self.whole_tree())
            .cast::<AnyObject>()
    }

    /// The element at a point of the screen: an autoreleased element of the adapter.
    pub(crate) fn element_at(&self, screen_point: NSPoint) -> *mut AnyObject {
        let point = accesskit_macos::NSPoint::new(screen_point.x, screen_point.y);
        self.adapter
            .borrow_mut()
            .hit_test(point, &mut self.whole_tree())
            .cast::<AnyObject>()
    }

    /// A frame ran: tells the adapter what it changed, when an assistive technology reads
    /// the surface. The content is not borrowed by the caller.
    pub(crate) fn frame_ran(&self) {
        let events = self
            .adapter
            .borrow_mut()
            .update_if_active(|| self.content.borrow_mut().accessibility_update());
        // The adapter is free again before its events are raised: raising them may call back
        // into it.
        if let Some(events) = events {
            events.raise();
        }
    }

    /// The window of the view got or lost the keyboard.
    pub(crate) fn view_focus_changed(&self, is_view_focused: bool) {
        let events = self.adapter.borrow_mut().update_view_focus_state(is_view_focused);
        if let Some(events) = events {
            events.raise();
        }
    }

    fn whole_tree(&self) -> WholeTree<'_> {
        WholeTree { content: &self.content }
    }
}

impl Debug for SurfaceAccessibility {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        formatter
            .debug_struct("SurfaceAccessibility")
            .field("adapter", &self.adapter)
            .finish_non_exhaustive()
    }
}
