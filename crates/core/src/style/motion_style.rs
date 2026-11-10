use crate::style::declaration_source::DeclarationSource;

/// The longhands of `transition` and `animation` in a computed style, by the declarations that
/// won the cascade; a longhand no declaration sets keeps its initial value.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct MotionStyle {
    pub(crate) transition_property: Option<DeclarationSource>,
    pub(crate) transition_duration: Option<DeclarationSource>,
    pub(crate) transition_timing_function: Option<DeclarationSource>,
    pub(crate) transition_delay: Option<DeclarationSource>,
    pub(crate) animation_name: Option<DeclarationSource>,
    pub(crate) animation_duration: Option<DeclarationSource>,
    pub(crate) animation_timing_function: Option<DeclarationSource>,
    pub(crate) animation_delay: Option<DeclarationSource>,
    pub(crate) animation_iteration_count: Option<DeclarationSource>,
    pub(crate) animation_direction: Option<DeclarationSource>,
    pub(crate) animation_fill_mode: Option<DeclarationSource>,
}

impl MotionStyle {
    pub(crate) const INITIAL: Self = Self {
        transition_property: None,
        transition_duration: None,
        transition_timing_function: None,
        transition_delay: None,
        animation_name: None,
        animation_duration: None,
        animation_timing_function: None,
        animation_delay: None,
        animation_iteration_count: None,
        animation_direction: None,
        animation_fill_mode: None,
    };
}
