/// One filter of a group with its lengths resolved, as the renderer applies it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Filter {
    /// The standard deviation of the Gaussian, in CSS pixels.
    Blur(f32),
    Brightness(f32),
    Contrast(f32),
    Grayscale(f32),
    Saturate(f32),
    Opacity(f32),
}

impl Filter {
    /// How far past its content the filtered picture reaches: a Gaussian has faded out three
    /// standard deviations away; every other filter stays inside the content.
    pub fn outset(&self) -> f32 {
        match self {
            Self::Blur(deviation) => deviation * BLUR_REACH_PER_DEVIATION,
            _ => 0.0,
        }
    }
}

const BLUR_REACH_PER_DEVIATION: f32 = 3.0;
