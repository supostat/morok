use morok_style::Color;

const CHANNEL_MAXIMUM: f32 = 255.0;

/// The color as the frame buffer blends it: sRGB values, not linearized, multiplied by alpha.
pub(crate) fn premultiplied(color: Color) -> [f32; 4] {
    let channel = |value: u8| f32::from(value) / CHANNEL_MAXIMUM * color.alpha;
    [
        channel(color.red),
        channel(color.green),
        channel(color.blue),
        color.alpha,
    ]
}

#[cfg(test)]
mod tests {
    use morok_style::Color;

    use super::premultiplied;

    fn as_bytes(channels: [f32; 4]) -> [u8; 4] {
        channels.map(|channel| (channel * 255.0).round() as u8)
    }

    #[test]
    fn the_color_channels_are_scaled_by_alpha() {
        let color = Color {
            red: 255,
            green: 0,
            blue: 102,
            alpha: 0.5,
        };

        assert_eq!(as_bytes(premultiplied(color)), [128, 0, 51, 128]);
    }

    #[test]
    fn an_opaque_color_keeps_its_channels() {
        let color = Color {
            red: 20,
            green: 143,
            blue: 120,
            alpha: 1.0,
        };

        assert_eq!(as_bytes(premultiplied(color)), [20, 143, 120, 255]);
    }
}
