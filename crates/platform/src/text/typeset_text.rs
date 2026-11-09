use std::ptr;

use morok_core::{TextExtent, WidthConstraint};
use objc2_core_foundation::{
    CFAttributedString, CFDictionary, CFIndex, CFNumber, CFRange, CFRetained, CFString, CFStringTokenizer, CFType,
    kCFStringTokenizerUnitLineBreak,
};
use objc2_core_text::{
    CTFont, CTLine, CTTypesetter, kCTFontAttributeName, kCTKernAttributeName, kCTLigatureAttributeName,
};

/// A line as wide as its constraint must fit it; widths come out of sums of floats.
const FIT_TOLERANCE: f64 = 0.01;
/// Wider than any line: a constraint that breaks lines only where the text does.
const UNBOUNDED_WIDTH: f64 = 1.0e9;
const ELLIPSIS: &str = "\u{2026}";

/// A text ready to be broken into lines: its attributed string, the typesetter of Core Text
/// over it, and the places a line may break.
pub(crate) struct TypesetText {
    code_units: Vec<u16>,
    attributes: CFRetained<CFDictionary<CFString, CFType>>,
    styled_text: CFRetained<CFAttributedString>,
    typesetter: CFRetained<CTTypesetter>,
    /// The offsets, in UTF-16 code units, a line may end at, in rising order; the end of the
    /// text is the last.
    break_opportunities: Vec<CFIndex>,
}

impl TypesetText {
    /// CSS letter spacing follows every character and takes ligatures apart; without it the
    /// font keeps its own kerning and ligatures.
    pub(crate) fn new(text: &str, font: &CTFont, letter_spacing: f32) -> Self {
        // SAFETY: the keys are constants of Core Text.
        let (font_key, kern_key, ligature_key) =
            unsafe { (kCTFontAttributeName, kCTKernAttributeName, kCTLigatureAttributeName) };
        let spacing = CFNumber::new_f64(f64::from(letter_spacing));
        let no_ligatures = CFNumber::new_i32(0);
        let (font, spacing, no_ligatures): (&CFType, &CFType, &CFType) =
            (font.as_ref(), spacing.as_ref(), no_ligatures.as_ref());
        let (keys, values): (&[&CFString], &[&CFType]) = if letter_spacing == 0.0 {
            (&[font_key], &[font])
        } else {
            (&[font_key, kern_key, ligature_key], &[font, spacing, no_ligatures])
        };
        let attributes = CFDictionary::<CFString, CFType>::from_slices(keys, values);
        let string = CFString::from_str(text);
        let styled_text = styled(&string, &attributes);
        // SAFETY: the attributed string is valid.
        let typesetter = unsafe { CTTypesetter::with_attributed_string(&styled_text) };
        Self {
            code_units: text.encode_utf16().collect(),
            break_opportunities: break_opportunities(&string),
            attributes,
            styled_text,
            typesetter,
        }
    }

    /// Calls `each` with every line of the text, top to bottom. Lines break where the text
    /// has a line break and, under a definite width, where a line is full; a word wider than
    /// the constraint stays whole and overflows, as CSS has it.
    ///
    /// # Panics
    ///
    /// Under the min-content constraint: its lines are measured, never set.
    pub(crate) fn for_each_line(&self, width: WidthConstraint, mut each: impl FnMut(&CTLine)) {
        let width = match width {
            WidthConstraint::Definite(width) => f64::from(width) + FIT_TOLERANCE,
            WidthConstraint::MaxContent => UNBOUNDED_WIDTH,
            WidthConstraint::MinContent => panic!("the lines of a text are not set at its min-content width"),
        };
        let length = string_index(self.code_units.len());
        let mut start = 0;
        while start < length {
            // SAFETY: `start` lies inside the string of the typesetter.
            let suggested = start + unsafe { self.typesetter.suggest_line_break(start, width) };
            let end = if self.break_opportunities.binary_search(&suggested).is_ok() {
                suggested
            } else {
                self.next_break_opportunity(start)
            };
            // SAFETY: the range lies inside the string of the typesetter.
            let line = unsafe {
                self.typesetter.line(CFRange {
                    location: start,
                    length: end - start,
                })
            };
            each(&line);
            start = end;
        }
    }

    /// The widest stretch of the text no line break may split, and the number of stretches:
    /// the text at its min-content width.
    pub(crate) fn unbreakable_extent(&self) -> TextExtent {
        // SAFETY: the attributed string is valid.
        let line = unsafe { CTLine::with_attributed_string(&self.styled_text) };
        // SAFETY: the offsets lie inside the string of the line.
        let offset = |index: CFIndex| unsafe { line.offset_for_string_index(index, ptr::null_mut()) };
        let mut extent = TextExtent::default();
        let mut start = 0;
        for &opportunity in &self.break_opportunities {
            let end = opportunity as usize;
            let visible_end = self.code_units[start..end]
                .iter()
                .rposition(|unit| !is_white_space(*unit))
                .map_or(start, |last| start + last + 1);
            let advance = offset(string_index(visible_end)) - offset(string_index(start));
            extent.width = extent.width.max(advance.abs() as f32);
            extent.line_count += 1;
            start = end;
        }
        extent
    }

    /// The line an ellipsis is set in, in the font and spacing of the text.
    pub(crate) fn ellipsis_line(&self) -> CFRetained<CTLine> {
        let ellipsis = styled(&CFString::from_str(ELLIPSIS), &self.attributes);
        // SAFETY: the attributed string is valid.
        unsafe { CTLine::with_attributed_string(&ellipsis) }
    }

    fn next_break_opportunity(&self, after: CFIndex) -> CFIndex {
        let position = self
            .break_opportunities
            .partition_point(|opportunity| *opportunity <= after);
        self.break_opportunities[position]
    }
}

fn styled(string: &CFString, attributes: &CFDictionary<CFString, CFType>) -> CFRetained<CFAttributedString> {
    // SAFETY: the attributes are string attributes of Core Text.
    unsafe { CFAttributedString::new(None, Some(string), Some(attributes.as_opaque())) }
        .expect("Core Foundation creates an attributed string")
}

/// Where Unicode line breaking lets a line end.
fn break_opportunities(string: &CFString) -> Vec<CFIndex> {
    let whole = CFRange {
        location: 0,
        length: string.length(),
    };
    // SAFETY: the range is that of the string.
    let tokenizer = unsafe { CFStringTokenizer::new(None, Some(string), whole, kCFStringTokenizerUnitLineBreak, None) }
        .expect("Core Foundation creates a tokenizer");
    let mut opportunities = Vec::new();
    while !tokenizer.advance_to_next_token().is_empty() {
        let token = tokenizer.current_token_range();
        opportunities.push(token.location + token.length);
    }
    if opportunities.last() != Some(&whole.length) {
        opportunities.push(whole.length);
    }
    opportunities
}

/// An offset into a text as Core Foundation counts it.
fn string_index(offset: usize) -> CFIndex {
    CFIndex::try_from(offset).expect("a text is shorter than the largest index of Core Foundation")
}

fn is_white_space(code_unit: u16) -> bool {
    char::from_u32(u32::from(code_unit)).is_some_and(char::is_whitespace)
}
