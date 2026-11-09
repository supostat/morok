/// A color in CIELAB under the D65 white point, the space CIEDE2000 measures differences in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LabColor {
    pub(crate) lightness: f64,
    pub(crate) a: f64,
    pub(crate) b: f64,
}

const D65_WHITE: [f64; 3] = [0.950_47, 1.0, 1.088_83];
const SRGB_TO_XYZ: [[f64; 3]; 3] = [
    [0.412_456_4, 0.357_576_1, 0.180_437_5],
    [0.212_672_9, 0.715_152_2, 0.072_175_0],
    [0.019_333_9, 0.119_192_0, 0.950_304_1],
];
/// The chroma, to the seventh power, around which CIEDE2000 stops rotating the hue of blues.
const CHROMA_KNEE: f64 = 6_103_515_625.0;

impl LabColor {
    /// The color of opaque sRGB channels.
    pub(crate) fn from_srgb([red, green, blue]: [u8; 3]) -> Self {
        let linear = [red, green, blue].map(linear_channel);
        let [x, y, z] = [0, 1, 2].map(|row| {
            let weights = SRGB_TO_XYZ[row];
            let tristimulus = weights[0] * linear[0] + weights[1] * linear[1] + weights[2] * linear[2];
            lab_function(tristimulus / D65_WHITE[row])
        });
        Self {
            lightness: 116.0 * y - 16.0,
            a: 500.0 * (x - y),
            b: 200.0 * (y - z),
        }
    }

    /// The CIEDE2000 difference between the two colors: about 1 is the smallest difference an
    /// eye tells, whatever the colors are.
    pub(crate) fn difference(self, other: Self) -> f64 {
        let mean_chroma = f64::midpoint(self.a.hypot(self.b), other.a.hypot(other.b));
        let chroma_boost = 1.5 - 0.5 * knee_ratio(mean_chroma).sqrt();
        let (own_a, other_a) = (self.a * chroma_boost, other.a * chroma_boost);
        let (own_chroma, other_chroma) = (own_a.hypot(self.b), other_a.hypot(other.b));
        let (own_hue, other_hue) = (hue_degrees(self.b, own_a), hue_degrees(other.b, other_a));
        let is_gray = own_chroma * other_chroma == 0.0;

        let lightness_delta = other.lightness - self.lightness;
        let chroma_delta = other_chroma - own_chroma;
        let hue_angle_delta = match other_hue - own_hue {
            _ if is_gray => 0.0,
            delta if delta > 180.0 => delta - 360.0,
            delta if delta < -180.0 => delta + 360.0,
            delta => delta,
        };
        let hue_delta = 2.0 * (own_chroma * other_chroma).sqrt() * (hue_angle_delta / 2.0).to_radians().sin();

        let mean_lightness = f64::midpoint(self.lightness, other.lightness);
        let mean_chroma = f64::midpoint(own_chroma, other_chroma);
        let hue_sum = own_hue + other_hue;
        let mean_hue = match (own_hue - other_hue).abs() {
            _ if is_gray => hue_sum,
            distance if distance <= 180.0 => hue_sum / 2.0,
            _ if hue_sum < 360.0 => hue_sum / 2.0 + 180.0,
            _ => hue_sum / 2.0 - 180.0,
        };
        let hue_weight = 1.0 - 0.17 * cosine_degrees(mean_hue - 30.0)
            + 0.24 * cosine_degrees(2.0 * mean_hue)
            + 0.32 * cosine_degrees(3.0 * mean_hue + 6.0)
            - 0.20 * cosine_degrees(4.0 * mean_hue - 63.0);
        let lightness_offset = (mean_lightness - 50.0).powi(2);
        let lightness_scale = 1.0 + 0.015 * lightness_offset / (20.0 + lightness_offset).sqrt();
        let chroma_scale = 1.0 + 0.045 * mean_chroma;
        let hue_scale = 1.0 + 0.015 * mean_chroma * hue_weight;
        let rotation_angle = 30.0 * (-((mean_hue - 275.0) / 25.0).powi(2)).exp();
        let rotation = -(2.0 * rotation_angle).to_radians().sin() * 2.0 * knee_ratio(mean_chroma).sqrt();

        let lightness_term = lightness_delta / lightness_scale;
        let chroma_term = chroma_delta / chroma_scale;
        let hue_term = hue_delta / hue_scale;
        (lightness_term.powi(2) + chroma_term.powi(2) + hue_term.powi(2) + rotation * chroma_term * hue_term).sqrt()
    }
}

fn linear_channel(channel: u8) -> f64 {
    let encoded = f64::from(channel) / 255.0;
    if encoded <= 0.040_45 {
        encoded / 12.92
    } else {
        ((encoded + 0.055) / 1.055).powf(2.4)
    }
}

fn lab_function(ratio: f64) -> f64 {
    const LINEAR_BELOW: f64 = 216.0 / 24_389.0;
    const SLOPE: f64 = 24_389.0 / 27.0;
    if ratio > LINEAR_BELOW {
        ratio.cbrt()
    } else {
        (SLOPE * ratio + 16.0) / 116.0
    }
}

/// How far the chroma is past the knee, from 0 for a gray to 1 for a saturated color.
fn knee_ratio(chroma: f64) -> f64 {
    let seventh_power = chroma.powi(7);
    seventh_power / (seventh_power + CHROMA_KNEE)
}

/// The hue angle from 0 to 360 degrees; a gray has the hue 0.
fn hue_degrees(b: f64, a: f64) -> f64 {
    if a == 0.0 && b == 0.0 {
        return 0.0;
    }
    let degrees = b.atan2(a).to_degrees();
    if degrees < 0.0 { degrees + 360.0 } else { degrees }
}

fn cosine_degrees(degrees: f64) -> f64 {
    degrees.to_radians().cos()
}

#[cfg(test)]
mod tests {
    use super::LabColor;

    /// Pairs of the test data published with the formula by Sharma, Wu and Dalal: the two
    /// colors and their difference.
    const PUBLISHED_DIFFERENCES: [([f64; 3], [f64; 3], f64); 12] = [
        ([50.0, 2.6772, -79.7751], [50.0, 0.0, -82.7485], 2.0425),
        ([50.0, 3.1571, -77.2803], [50.0, 0.0, -82.7485], 2.8615),
        ([50.0, 2.8361, -74.0200], [50.0, 0.0, -82.7485], 3.4412),
        ([50.0, -1.3802, -84.2814], [50.0, 0.0, -82.7485], 1.0000),
        ([50.0, 0.0, 0.0], [50.0, -1.0, 2.0], 2.3669),
        ([50.0, 2.49, -0.001], [50.0, -2.49, 0.0009], 7.1792),
        ([50.0, 2.49, -0.001], [50.0, -2.49, 0.0011], 7.2195),
        ([50.0, -0.001, 2.49], [50.0, 0.0009, -2.49], 4.8045),
        ([50.0, 2.5, 0.0], [50.0, 0.0, -2.5], 4.3065),
        ([50.0, 2.5, 0.0], [73.0, 25.0, -18.0], 27.1492),
        ([60.2574, -34.0099, 36.2677], [60.4626, -34.1751, 39.4387], 1.2644),
        ([2.0776, 0.0795, -1.1350], [0.9033, -0.0636, -0.5514], 0.9082),
    ];

    fn lab([lightness, a, b]: [f64; 3]) -> LabColor {
        LabColor { lightness, a, b }
    }

    #[test]
    fn the_difference_equals_the_published_test_data_in_both_directions() {
        for (first, second, published) in PUBLISHED_DIFFERENCES {
            for difference in [lab(first).difference(lab(second)), lab(second).difference(lab(first))] {
                assert!(
                    (difference - published).abs() < 0.000_1,
                    "{first:?} and {second:?} differ by {difference}, published as {published}"
                );
            }
        }
    }

    #[test]
    fn the_ends_of_the_gray_axis_are_black_and_white() {
        let black = LabColor::from_srgb([0, 0, 0]);
        let white = LabColor::from_srgb([255, 255, 255]);

        assert!(black.lightness.abs() < 0.001, "{black:?}");
        assert!((white.lightness - 100.0).abs() < 0.001, "{white:?}");
        assert!(white.a.abs() < 0.01 && white.b.abs() < 0.01, "{white:?}");
        assert!((black.difference(white) - 100.0).abs() < 0.01);
    }

    #[test]
    fn a_saturated_color_lands_where_the_srgb_standard_puts_it() {
        let red = LabColor::from_srgb([255, 0, 0]);

        assert!((red.lightness - 53.24).abs() < 0.01, "{red:?}");
        assert!((red.a - 80.09).abs() < 0.01, "{red:?}");
        assert!((red.b - 67.20).abs() < 0.01, "{red:?}");
    }
}
