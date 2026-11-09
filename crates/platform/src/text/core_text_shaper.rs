use std::ptr::{self, NonNull};

use morok_core::{
    FontId, FontMetrics, FontRequest, ItemRange, ShapedGlyph, ShapedLine, ShapedRun, ShapedText, TextExtent,
    TextRequest, TextShaper, WidthConstraint,
};
use objc2_core_foundation::{CFArray, CFDictionary, CFRange, CFRetained, CFString, CGPoint, Type};
use objc2_core_graphics::CGGlyph;
use objc2_core_text::{CTFont, CTLine, CTLineTruncationType, CTRun, kCTFontAttributeName};

use crate::text::font_library::font_library;
use crate::text::measure_cache::{MeasureCache, MeasureKey};
use crate::text::typeset_text::TypesetText;

/// A line as wide as the width it is cut to needs no ellipsis; widths come out of sums of floats.
const FIT_TOLERANCE: f32 = 0.01;

/// The text shaper of the kit: Core Text picks the fonts, shapes the glyphs, orders
/// bidirectional runs and breaks the lines. Measurements are remembered by string, style and
/// width.
#[derive(Debug, Default)]
pub struct CoreTextShaper {
    measurements: MeasureCache,
    /// Scratch space for the glyphs and positions of a run.
    run_glyphs: Vec<CGGlyph>,
    run_positions: Vec<CGPoint>,
}

impl CoreTextShaper {
    pub fn new() -> Self {
        Self::default()
    }

    /// How many measurements the shaper remembers.
    pub fn cached_measurements(&self) -> usize {
        self.measurements.len()
    }

    /// Appends the runs of a line Core Text set, left to right as they are drawn.
    fn append_line(&mut self, line: &CTLine, shaped: &mut ShapedText) {
        let runs_start = shaped.runs.len();
        // SAFETY: the glyph runs of a line are an array of `CTRun`.
        let runs = unsafe { CFRetained::cast_unchecked::<CFArray<CTRun>>(line.glyph_runs()) };
        for run in runs.iter() {
            let glyphs_start = shaped.glyphs.len();
            // SAFETY: the run is valid.
            let glyph_count = unsafe { run.glyph_count() } as usize;
            self.run_glyphs.resize(glyph_count, 0);
            self.run_positions.resize(glyph_count, CGPoint { x: 0.0, y: 0.0 });
            if glyph_count == 0 {
                continue;
            }
            let whole_run = CFRange { location: 0, length: 0 };
            // SAFETY: the buffers hold as many entries as the run has glyphs.
            unsafe {
                run.glyphs(whole_run, NonNull::from(&mut self.run_glyphs[0]));
                run.positions(whole_run, NonNull::from(&mut self.run_positions[0]));
            }
            shaped.glyphs.extend(
                self.run_glyphs
                    .iter()
                    .zip(&self.run_positions)
                    .map(|(glyph, position)| ShapedGlyph {
                        glyph: *glyph,
                        x: position.x as f32,
                        y: -position.y as f32,
                    }),
            );
            shaped.runs.push(ShapedRun {
                font: run_font(&run),
                glyphs: ItemRange {
                    start: glyphs_start as u32,
                    end: shaped.glyphs.len() as u32,
                },
            });
        }
        shaped.lines.push(ShapedLine {
            width: line_width(line),
            runs: ItemRange {
                start: runs_start as u32,
                end: shaped.runs.len() as u32,
            },
        });
    }
}

impl TextShaper for CoreTextShaper {
    /// Ascent, descent and line gap are whole pixels: browsers round them, and a baseline
    /// computed from fractions would sit off theirs.
    fn font_metrics(&mut self, font: &FontRequest<'_>) -> FontMetrics {
        let (_, font) = resolved(font);
        // SAFETY: the font is valid.
        let (ascent, descent, line_gap, underline_position, thickness, x_height) = unsafe {
            (
                font.ascent(),
                font.descent(),
                font.leading(),
                font.underline_position(),
                font.underline_thickness(),
                font.x_height(),
            )
        };
        FontMetrics {
            ascent: ascent.round() as f32,
            descent: descent.round() as f32,
            line_gap: line_gap.round() as f32,
            underline_top: (-underline_position - thickness / 2.0) as f32,
            line_through_top: (-x_height / 2.0 - thickness / 2.0) as f32,
            decoration_thickness: thickness as f32,
        }
    }

    fn measure(&mut self, request: &TextRequest<'_>) -> TextExtent {
        if request.text.is_empty() {
            return TextExtent::default();
        }
        let (font_id, font) = resolved(&request.font);
        let key = MeasureKey::new(font_id, request.letter_spacing, request.width);
        if let Some(extent) = self.measurements.get(key, request.text) {
            return extent;
        }
        let typeset = TypesetText::new(request.text, &font, request.letter_spacing);
        let extent = if request.width == WidthConstraint::MinContent {
            typeset.unbreakable_extent()
        } else {
            let mut extent = TextExtent::default();
            typeset.for_each_line(request.width, |line| {
                extent.width = extent.width.max(line_width(line));
                extent.line_count += 1;
            });
            extent
        };
        self.measurements.insert(key, request.text, extent);
        extent
    }

    fn shape(&mut self, request: &TextRequest<'_>, ellipsis_width: Option<f32>, shaped: &mut ShapedText) {
        shaped.clear();
        if request.text.is_empty() {
            return;
        }
        let (_, font) = resolved(&request.font);
        let typeset = TypesetText::new(request.text, &font, request.letter_spacing);
        typeset.for_each_line(request.width, |line| {
            let cut = ellipsis_width
                .filter(|width| line_width(line) > width + FIT_TOLERANCE)
                .and_then(|width| {
                    // SAFETY: the line and the ellipsis line are valid.
                    unsafe {
                        line.truncated_line(
                            f64::from(width),
                            CTLineTruncationType::End,
                            Some(&typeset.ellipsis_line()),
                        )
                    }
                });
            self.append_line(cut.as_deref().unwrap_or(line), shaped);
        });
    }
}

/// The font of a request and its name in the font library.
fn resolved(request: &FontRequest<'_>) -> (FontId, CFRetained<CTFont>) {
    let mut library = font_library();
    let font = library.resolve(request);
    (font, library.font(font).retain())
}

/// The font Core Text set a run in: the font of the text, or the fallback it picked for
/// characters the font lacks.
fn run_font(run: &CTRun) -> FontId {
    // SAFETY: the attributes of a run hold its font under the font key.
    let font = unsafe {
        let attributes = CFRetained::cast_unchecked::<CFDictionary<CFString, CTFont>>(run.attributes());
        attributes.get(kCTFontAttributeName)
    }
    .expect("Core Text attaches a font to every run");
    font_library().name(&font)
}

/// The advance of a line without the white space it ends with.
fn line_width(line: &CTLine) -> f32 {
    // SAFETY: the line is valid; null pointers skip the metrics that are not asked for.
    let (advance, trailing_white_space) = unsafe {
        (
            line.typographic_bounds(ptr::null_mut(), ptr::null_mut(), ptr::null_mut()),
            line.trailing_whitespace_width(),
        )
    };
    (advance - trailing_white_space).max(0.0) as f32
}
