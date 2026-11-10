#![allow(dead_code)]

use std::time::Duration;

use morok_core::{
    ElementId, ElementTree, FontId, FontMetrics, FontRequest, InputHandler, ItemRange, Key, KeyEvent, KeyPhase,
    Modifiers, Point, PointerButton, PointerEvent, PointerEventKind, PointerInput, PointerInputKind, Propagation, Rect,
    Role, ShapedGlyph, ShapedLine, ShapedRun, ShapedText, ShortcutId, Size, TextExtent, TextRequest, TextShaper,
    WidthConstraint,
};
use morok_css::{CompiledTokens, SourceFile, TokenId, compile_stylesheets, compile_tokens};
use morok_style::{Class, Color, FontStyle, FontVariantNumeric, FontWeight, StyleSheet, Theme};

pub(crate) const BASE_TOKENS_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../design/system/tokens.css");
pub(crate) const VIEWPORT: Size = Size {
    width: 800.0,
    height: 600.0,
};

/// The advance of a character of the monospace test font, per CSS pixel of font size.
pub(crate) const ADVANCE_PER_FONT_SIZE: f32 = 0.625;
pub(crate) const ASCENT_PER_FONT_SIZE: f32 = 0.9375;
pub(crate) const DESCENT_PER_FONT_SIZE: f32 = 0.3125;
pub(crate) const UNDERLINE_TOP_PER_FONT_SIZE: f32 = 0.125;
pub(crate) const LINE_THROUGH_TOP_PER_FONT_SIZE: f32 = -0.375;
pub(crate) const DECORATION_THICKNESS_PER_FONT_SIZE: f32 = 0.0625;
/// At the initial font size of 16 pixels a character is 10 pixels wide and a line 20 high.
pub(crate) const CHARACTER_WIDTH: f32 = 10.0;
pub(crate) const LINE_HEIGHT: f32 = 20.0;
pub(crate) const ELLIPSIS: char = '\u{2026}';

/// What a shaper was asked about a font.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RequestedFont {
    pub(crate) families: Vec<String>,
    pub(crate) size: f32,
    pub(crate) weight: FontWeight,
    pub(crate) style: FontStyle,
    pub(crate) numeric: FontVariantNumeric,
}

/// Sets text as a monospace font would: every character as wide as the next, a line break
/// between words where a line is full and at every line break of the text. The glyph of a
/// character is its code. It remembers what it was asked.
#[derive(Debug, Default)]
pub(crate) struct MonospaceShaper {
    pub(crate) measured_texts: Vec<String>,
    pub(crate) shaped_texts: Vec<String>,
    pub(crate) last_font: Option<RequestedFont>,
}

impl MonospaceShaper {
    fn advance(request: &TextRequest<'_>) -> f32 {
        request.font.size * ADVANCE_PER_FONT_SIZE + request.letter_spacing
    }

    /// The lines of the text as ranges of characters, white space at the end of a wrapped
    /// line left out.
    fn lines(request: &TextRequest<'_>) -> Vec<String> {
        let advance = Self::advance(request);
        let mut lines = Vec::new();
        for paragraph in request.text.split('\n') {
            if request.text.is_empty() {
                break;
            }
            let fits = |characters: usize| match request.width {
                WidthConstraint::MaxContent => true,
                WidthConstraint::MinContent => false,
                WidthConstraint::Definite(width) => characters as f32 * advance <= width,
            };
            let mut words = paragraph.split(' ');
            let mut line = words.next().unwrap_or_default().to_owned();
            for word in words {
                let extended = format!("{line} {word}");
                if !line.trim_end().is_empty() && !fits(extended.trim_end().chars().count()) {
                    lines.push(line.trim_end().to_owned());
                    word.clone_into(&mut line);
                } else {
                    line = extended;
                }
            }
            lines.push(line);
        }
        lines
    }

    fn remember_font(&mut self, font: &FontRequest<'_>) {
        self.last_font = Some(RequestedFont {
            families: font.families.iter().map(ToString::to_string).collect(),
            size: font.size,
            weight: font.weight,
            style: font.style,
            numeric: font.numeric,
        });
    }
}

impl TextShaper for MonospaceShaper {
    fn font_metrics(&mut self, font: &FontRequest<'_>) -> FontMetrics {
        FontMetrics {
            ascent: font.size * ASCENT_PER_FONT_SIZE,
            descent: font.size * DESCENT_PER_FONT_SIZE,
            line_gap: 0.0,
            underline_top: font.size * UNDERLINE_TOP_PER_FONT_SIZE,
            line_through_top: font.size * LINE_THROUGH_TOP_PER_FONT_SIZE,
            decoration_thickness: font.size * DECORATION_THICKNESS_PER_FONT_SIZE,
        }
    }

    fn measure(&mut self, request: &TextRequest<'_>) -> TextExtent {
        self.measured_texts.push(request.text.to_owned());
        self.remember_font(&request.font);
        let lines = Self::lines(request);
        let widest = lines.iter().map(|line| line.chars().count()).max().unwrap_or(0);
        TextExtent {
            width: widest as f32 * Self::advance(request),
            line_count: lines.len(),
        }
    }

    fn shape(&mut self, request: &TextRequest<'_>, ellipsis_width: Option<f32>, shaped: &mut ShapedText) {
        self.shaped_texts.push(request.text.to_owned());
        self.remember_font(&request.font);
        let advance = Self::advance(request);
        shaped.clear();
        for line in Self::lines(request) {
            let mut characters: Vec<char> = line.chars().collect();
            let overflows = |count: usize| ellipsis_width.is_some_and(|width| count as f32 * advance > width);
            if overflows(characters.len()) {
                while !characters.is_empty() && overflows(characters.len() + 1) {
                    characters.pop();
                }
                characters.push(ELLIPSIS);
            }
            let glyphs_start = shaped.glyphs.len() as u32;
            shaped
                .glyphs
                .extend(characters.iter().enumerate().map(|(position, character)| ShapedGlyph {
                    glyph: *character as u16,
                    x: position as f32 * advance,
                    y: 0.0,
                }));
            let runs_start = shaped.runs.len() as u32;
            shaped.runs.push(ShapedRun {
                font: FontId(0),
                glyphs: ItemRange {
                    start: glyphs_start,
                    end: shaped.glyphs.len() as u32,
                },
            });
            shaped.lines.push(ShapedLine {
                width: characters.len() as f32 * advance,
                runs: ItemRange {
                    start: runs_start,
                    end: shaped.runs.len() as u32,
                },
            });
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
    pub(crate) text_shaper: MonospaceShaper,
    pub(crate) viewport: Size,
    /// Device pixels to a CSS pixel on the surface the scene is laid out for.
    pub(crate) scale: f32,
    /// The time of the last frame.
    pub(crate) time: Duration,
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
            text_shaper: MonospaceShaper::default(),
            viewport: VIEWPORT,
            scale: 1.0,
            time: Duration::ZERO,
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

    /// Runs the three stages of a frame at the time of the frame before.
    pub(crate) fn frame(&mut self) -> Frame {
        self.frame_at(self.time)
    }

    /// Runs the three stages of a frame at `time`.
    pub(crate) fn frame_at(&mut self, time: Duration) -> Frame {
        self.time = time;
        let resolved = self.tree.resolve_styles(time);
        let laid_out = self
            .tree
            .compute_layout(self.viewport, self.scale, &mut self.text_shaper);
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

    /// Gives the tree what the pointer did at a point of the surface, with no modifier held.
    pub(crate) fn pointer(&mut self, kind: PointerInputKind, x: f32, y: f32, log: &mut InputLog) {
        let input = PointerInput {
            kind,
            position: Point { x, y },
            modifiers: Modifiers::NONE,
        };
        self.tree.pointer_input(&input, log);
    }

    pub(crate) fn move_pointer(&mut self, x: f32, y: f32, log: &mut InputLog) {
        self.pointer(PointerInputKind::Moved, x, y, log);
    }

    /// Presses the primary button once.
    pub(crate) fn press(&mut self, x: f32, y: f32, log: &mut InputLog) {
        let pressed = PointerInputKind::Pressed {
            button: PointerButton::Primary,
            click_count: 1,
        };
        self.pointer(pressed, x, y, log);
    }

    /// Releases the primary button after one press.
    pub(crate) fn release(&mut self, x: f32, y: f32, log: &mut InputLog) {
        let released = PointerInputKind::Released {
            button: PointerButton::Primary,
            click_count: 1,
        };
        self.pointer(released, x, y, log);
    }

    /// Presses a key once. Returns whether the tree used the key.
    pub(crate) fn press_key(&mut self, key: Key, modifiers: Modifiers, log: &mut InputLog) -> bool {
        let event = KeyEvent {
            phase: KeyPhase::Pressed,
            key,
            modifiers,
            is_repeat: false,
        };
        self.tree.key_input(&event, log)
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

/// Remembers the input a tree routed to its elements.
#[derive(Debug, Default)]
pub(crate) struct InputLog {
    pub(crate) pointer_events: Vec<(ElementId, PointerEventKind)>,
    pub(crate) key_events: Vec<(ElementId, KeyPhase, Key)>,
    pub(crate) shortcuts: Vec<ShortcutId>,
    /// The elements that use every event routed to them.
    pub(crate) consumers: Vec<ElementId>,
}

impl InputLog {
    /// The elements that heard of a pointer event of the kind, in the order they heard of it.
    pub(crate) fn receivers_of(&self, kind: PointerEventKind) -> Vec<ElementId> {
        self.pointer_events
            .iter()
            .filter(|(_, heard)| *heard == kind)
            .map(|(element, _)| *element)
            .collect()
    }

    /// The elements that were offered a key, in the order they were offered it.
    pub(crate) fn key_receivers(&self) -> Vec<ElementId> {
        self.key_events.iter().map(|(element, _, _)| *element).collect()
    }

    fn answer(&self, element: ElementId) -> Propagation {
        if self.consumers.contains(&element) {
            Propagation::Stop
        } else {
            Propagation::Continue
        }
    }
}

impl InputHandler for InputLog {
    fn pointer_event(&mut self, element: ElementId, event: &PointerEvent) -> Propagation {
        self.pointer_events.push((element, event.kind));
        self.answer(element)
    }

    fn key_event(&mut self, element: ElementId, event: &KeyEvent) -> Propagation {
        self.key_events.push((element, event.phase, event.key));
        self.answer(element)
    }

    fn shortcut(&mut self, shortcut: ShortcutId) {
        self.shortcuts.push(shortcut);
    }
}
