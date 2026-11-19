use morok_core::OverlayInput;
use objc2::rc::Retained;
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send};
use objc2_app_kit::{
    NSBackingStoreType, NSColor, NSPanel, NSResponder, NSWindow, NSWindowAnimationBehavior, NSWindowStyleMask,
};
use objc2_foundation::NSRect;

pub(crate) struct OverlayPanelState {
    input: OverlayInput,
}

define_class!(
    /// The window of an overlay: no frame, no background and no shadow of its own, so that
    /// all it shows is what its surface draws. It becomes the key window only for an overlay
    /// that takes the keyboard, and the application stays the one it was.
    #[unsafe(super(NSPanel, NSWindow, NSResponder))]
    #[thread_kind = MainThreadOnly]
    #[ivars = OverlayPanelState]
    pub(crate) struct OverlayPanel;

    impl OverlayPanel {
        #[unsafe(method(canBecomeKeyWindow))]
        fn can_become_key_window(&self) -> bool {
            self.ivars().input.takes_keyboard()
        }
    }
);

impl OverlayPanel {
    pub(crate) fn new(main_thread: MainThreadMarker, frame: NSRect, input: OverlayInput) -> Retained<Self> {
        let panel = Self::alloc(main_thread).set_ivars(OverlayPanelState { input });
        let style = NSWindowStyleMask::Borderless | NSWindowStyleMask::NonactivatingPanel;
        // SAFETY: the superclass initializes a window with these four arguments, and the
        // panel is not released when closed: the overlay that opened it owns it.
        let panel: Retained<Self> = unsafe {
            let panel: Retained<Self> = msg_send![
                super(panel),
                initWithContentRect: frame,
                styleMask: style,
                backing: NSBackingStoreType::Buffered,
                defer: false,
            ];
            panel.setReleasedWhenClosed(false);
            panel
        };
        panel.setOpaque(false);
        panel.setBackgroundColor(Some(&NSColor::clearColor()));
        panel.setHasShadow(false);
        panel.setIgnoresMouseEvents(!input.takes_pointer());
        panel.setHidesOnDeactivate(false);
        panel.setAnimationBehavior(NSWindowAnimationBehavior::None);
        panel
    }
}
