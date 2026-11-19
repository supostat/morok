use objc2::rc::{Retained, Weak};
use objc2::runtime::NSObject;
use objc2::{DefinedClass, MainThreadOnly, Message, define_class, msg_send};
use objc2_app_kit::NSMenuItem;

use crate::surface_view::SurfaceView;

pub(crate) struct MenuBarTargetState {
    /// The view whose menus the menu bar shows.
    view: Weak<SurfaceView>,
}

define_class!(
    /// What the lines of the menu bar send their action to: it tells the content of the view
    /// whose menus the bar shows which line the user chose.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = MenuBarTargetState]
    pub(crate) struct MenuBarTarget;

    impl MenuBarTarget {
        #[unsafe(method(menuBarCommandChosen:))]
        fn menu_bar_command_chosen(&self, item: &NSMenuItem) {
            let (Some(view), Ok(id)) = (self.ivars().view.load(), usize::try_from(item.tag())) else {
                return;
            };
            view.content().borrow_mut().menu_bar_command_chosen(id);
        }
    }
);

impl MenuBarTarget {
    pub(crate) fn new(view: &SurfaceView) -> Retained<Self> {
        let target = Self::alloc(view.mtm()).set_ivars(MenuBarTargetState {
            view: Weak::from_retained(&view.retain()),
        });
        // SAFETY: the superclass initializes an object with no arguments.
        unsafe { msg_send![super(target), init] }
    }
}
