use std::cell::Cell;

use objc2::rc::Retained;
use objc2::runtime::NSObject;
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send};
use objc2_app_kit::NSMenuItem;

pub(crate) struct MenuTargetState {
    chosen: Cell<Option<usize>>,
}

define_class!(
    /// What the lines of a menu of the system send their action to: it remembers the line
    /// the user chose.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = MenuTargetState]
    pub(crate) struct MenuTarget;

    impl MenuTarget {
        #[unsafe(method(commandChosen:))]
        fn command_chosen(&self, item: &NSMenuItem) {
            self.ivars().chosen.set(usize::try_from(item.tag()).ok());
        }
    }
);

impl MenuTarget {
    pub(crate) fn new(main_thread: MainThreadMarker) -> Retained<Self> {
        let target = Self::alloc(main_thread).set_ivars(MenuTargetState {
            chosen: Cell::new(None),
        });
        // SAFETY: the superclass initializes an object with no arguments.
        unsafe { msg_send![super(target), init] }
    }

    /// The `id` of the command the user chose; none when the menu was given up.
    pub(crate) fn chosen(&self) -> Option<usize> {
        self.ivars().chosen.get()
    }
}
