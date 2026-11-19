use std::time::Duration;

use morok_core::ElementId;
use morok_core::accesskit::Role as AccessibleRole;
use morok_reactive::Signal;

use crate::styles::CLASSES;
use crate::toast::Toast;
use crate::toast_tone::ToastTone;
use crate::view::Mount;

const LIFETIME: Duration = Duration::from_secs(4);

/// The toasts of a surface, stacked in its bottom right corner, the newest last: the ones the
/// signal holds. A toast leaves the signal by itself once its lifetime has passed.
#[derive(Debug)]
pub struct ToastStack {
    toasts: Signal<Vec<Toast>>,
    lifetime: Duration,
}

impl ToastStack {
    /// A stack whose toasts stay for four seconds.
    pub fn new(toasts: Signal<Vec<Toast>>) -> Self {
        Self {
            toasts,
            lifetime: LIFETIME,
        }
    }

    /// How long a toast stays, counted from the frame that first shows it.
    #[must_use]
    pub fn lifetime(mut self, lifetime: Duration) -> Self {
        self.lifetime = lifetime;
        self
    }

    /// Builds the stack. It lies in the corner of the box it is positioned against: mounted
    /// under the root, the corner of the surface.
    pub fn mount(self, mount: &mut Mount<'_>) -> ElementId {
        let Self { toasts, lifetime } = self;
        mount
            .element()
            .class(CLASSES.toasts)
            .for_each(
                toasts,
                |toast| toast.id,
                move |mount, toast| mount_toast(mount, toast, toasts, lifetime),
            )
            .id()
    }
}

/// Builds a toast: its dot and its lines. It takes itself out of `toasts` after `lifetime`.
fn mount_toast(mount: &mut Mount<'_>, toast: &Toast, toasts: Signal<Vec<Toast>>, lifetime: Duration) -> ElementId {
    let mut dot = mount.element();
    dot.class(CLASSES.toast_dot);
    match toast.tone {
        ToastTone::Success => {}
        ToastTone::Warning => {
            dot.class(CLASSES.warning);
        }
        ToastTone::Danger => {
            dot.class(CLASSES.danger);
        }
    }
    let dot = dot.id();
    let title = mount.text(toast.title.clone()).id();
    let detail = toast
        .detail
        .clone()
        .map(|detail| mount.text(detail).class(CLASSES.toast_detail).id());
    let mut lines = mount.element();
    lines.child(title);
    if let Some(detail) = detail {
        lines.child(detail);
    }
    let lines = lines.id();
    let announced = match &toast.detail {
        Some(detail) => format!("{}. {detail}", toast.title),
        None => toast.title.clone(),
    };
    let is_shown = mount.runtime().signal(true);
    let mut shown = mount.element();
    shown
        .class(CLASSES.toast)
        .accessible_role(AccessibleRole::Alert)
        .accessible_name(announced)
        .child(dot)
        .child(lines);
    let has_expired = shown.held_for(is_shown, lifetime);
    let shown = shown.id();
    let id = toast.id;
    mount.runtime().effect(move |runtime, _| {
        if runtime.get(has_expired) {
            runtime.update(toasts, |toasts| toasts.retain(|toast| toast.id != id));
        }
    });
    shown
}
