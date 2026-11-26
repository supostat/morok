use morok_core::accesskit::Role as AccessibleRole;
use morok_core::{ElementId, ImageId};

use crate::styles::CLASSES;
use crate::view::{Mount, Property};

/// A picture the application added: shown at its natural size unless the style of the element
/// sizes it, and fitted into that box by the `object-fit` of its style. While the picture
/// decodes, and when it is unreadable, the element is its box alone; an assistive technology
/// reads its label all the same.
#[derive(Debug)]
pub struct Image {
    image: Property<ImageId>,
    label: Property<String>,
}

impl Image {
    /// An image of the picture `image` names, which an assistive technology calls `label`.
    pub fn new(image: impl Into<Property<ImageId>>, label: impl Into<Property<String>>) -> Self {
        Self {
            image: image.into(),
            label: label.into(),
        }
    }

    /// Builds the picture: one element that shows it.
    pub fn mount(self, mount: &mut Mount<'_>) -> ElementId {
        mount
            .image(self.image)
            .class(CLASSES.image)
            .accessible_role(AccessibleRole::Image)
            .accessible_name(self.label)
            .id()
    }
}
