#![allow(dead_code)]

use morok_core::{ElementId, ElementTree, Rect, Role, Size, TextMeasureRequest, TextMeasurer, WidthConstraint};
use morok_css::{CompiledTokens, SourceFile, TokenId, compile_stylesheets, compile_tokens};
use morok_style::{Class, Color, StyleSheet, Theme};

pub(crate) const BASE_TOKENS_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../design/system/tokens.css");
pub(crate) const VIEWPORT: Size = Size {
    width: 800.0,
    height: 600.0,
};

pub(crate) const CHARACTER_WIDTH: f32 = 10.0;
pub(crate) const LINE_HEIGHT: f32 = 20.0;

/// Measures text as a monospace font would — every character as wide as the next, a line break
/// between words — and remembers every text it was asked about.
#[derive(Debug, Default)]
pub(crate) struct MonospaceMeasurer {
    pub(crate) measured_texts: Vec<String>,
}

impl MonospaceMeasurer {
    fn characters_per_line(text: &str, width: WidthConstraint) -> usize {
        let longest_word = text.split(' ').map(str::len).max().unwrap_or(0);
        match width {
            WidthConstraint::MinContent => longest_word,
            WidthConstraint::MaxContent => text.len(),
            WidthConstraint::Definite(width) => ((width / CHARACTER_WIDTH) as usize).clamp(longest_word, text.len()),
        }
    }

    fn line_count(text: &str, characters_per_line: usize) -> usize {
        let mut lines = 1;
        let mut line_length = 0;
        for word in text.split(' ') {
            let extended = if line_length == 0 {
                word.len()
            } else {
                line_length + 1 + word.len()
            };
            if extended > characters_per_line && line_length > 0 {
                lines += 1;
                line_length = word.len();
            } else {
                line_length = extended;
            }
        }
        lines
    }
}

impl TextMeasurer for MonospaceMeasurer {
    fn measure(&mut self, request: &TextMeasureRequest<'_>) -> Size {
        self.measured_texts.push(request.text.to_owned());
        let characters_per_line = Self::characters_per_line(request.text, request.width);
        Size {
            width: characters_per_line as f32 * CHARACTER_WIDTH,
            height: Self::line_count(request.text, characters_per_line) as f32 * LINE_HEIGHT,
        }
    }
}

/// How much work each stage of one frame did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Frame {
    pub(crate) resolved: usize,
    pub(crate) laid_out: usize,
    pub(crate) rebuilt: usize,
}

/// An element tree styled by a stylesheet the real compiler compiled against the base tokens.
pub(crate) struct Scene {
    pub(crate) tree: ElementTree,
    pub(crate) text_measurer: MonospaceMeasurer,
    pub(crate) viewport: Size,
    tokens: CompiledTokens,
    style_sheet: StyleSheet,
}

impl Scene {
    /// A scene under the base tokens of the kit.
    pub(crate) fn new(rules: &str) -> Self {
        let tokens = SourceFile::read(BASE_TOKENS_FILE).expect("the base tokens file is readable");
        Self::with_tokens(&tokens, rules)
    }

    /// A scene under tokens of its own; `rules` is the text of its stylesheet.
    pub(crate) fn with_tokens(tokens: &SourceFile, rules: &str) -> Self {
        let tokens = compile_tokens(tokens).unwrap_or_else(|diagnostics| panic!("the tokens compile:\n{diagnostics}"));
        let style_sheet = compile_stylesheets(&[SourceFile::new("scene.css", rules)], &tokens)
            .unwrap_or_else(|diagnostics| panic!("the stylesheet compiles:\n{diagnostics}"));
        Self {
            tree: ElementTree::new(style_sheet.clone(), tokens.light.clone()),
            text_measurer: MonospaceMeasurer::default(),
            viewport: VIEWPORT,
            tokens,
            style_sheet,
        }
    }

    pub(crate) fn class(&self, name: &str) -> Class {
        self.style_sheet
            .class_named(name)
            .unwrap_or_else(|| panic!("the stylesheet names the class `{name}`"))
    }

    pub(crate) fn root(&self) -> ElementId {
        self.tree.root()
    }

    /// Adds an element with the given classes as the last child of `parent`.
    pub(crate) fn add(&mut self, parent: ElementId, role: Role, class_names: &[&str]) -> ElementId {
        let element = self.tree.create(role);
        self.add_classes(element, class_names);
        self.tree.append_child(parent, element);
        element
    }

    pub(crate) fn add_text(&mut self, parent: ElementId, text: &str) -> ElementId {
        let element = self.add(parent, Role::Text, &[]);
        self.tree.set_text(element, text);
        element
    }

    pub(crate) fn add_classes(&mut self, element: ElementId, class_names: &[&str]) {
        for name in class_names {
            let class = self.class(name);
            self.tree.add_class(element, class);
        }
    }

    /// Runs the three stages of a frame.
    pub(crate) fn frame(&mut self) -> Frame {
        let resolved = self.tree.resolve_styles();
        let laid_out = self.tree.compute_layout(self.viewport, &mut self.text_measurer);
        let rebuilt = self.tree.build_display_list();
        Frame {
            resolved,
            laid_out,
            rebuilt,
        }
    }

    pub(crate) fn border_box(&self, element: ElementId) -> Rect {
        self.tree.border_box(element)
    }

    pub(crate) fn light_theme(&self) -> Theme {
        self.tokens.light.clone()
    }

    pub(crate) fn dark_theme(&self) -> Theme {
        self.tokens.dark.clone()
    }

    pub(crate) fn light_color(&self, token_name: &str) -> Color {
        *self.tokens.light.resolve(self.color_token(token_name))
    }

    pub(crate) fn dark_color(&self, token_name: &str) -> Color {
        *self.tokens.dark.resolve(self.color_token(token_name))
    }

    fn color_token(&self, token_name: &str) -> morok_style::Token<Color> {
        match self.tokens.id(token_name) {
            Some(TokenId::Color(token)) => token,
            other => panic!("`--{token_name}` is not a color token: {other:?}"),
        }
    }
}
