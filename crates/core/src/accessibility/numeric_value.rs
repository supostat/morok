/// A number an element stands for, a slider for one, with the range it moves in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NumericValue {
    pub value: f64,
    pub min: f64,
    pub max: f64,
    /// How far one increment or decrement moves the value.
    pub step: f64,
}
