use std::borrow::Cow;

use crate::shadow_layer::ShadowLayer;

/// Shadow layers in CSS order: the first layer is painted on top.
#[derive(Clone, Debug, PartialEq)]
pub struct Shadow {
    pub layers: Cow<'static, [ShadowLayer]>,
}
