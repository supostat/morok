use crate::paint::gradient::Gradient;

#[derive(Clone, Debug, PartialEq)]
pub enum BackgroundImage {
    None,
    Gradient(Gradient),
}
