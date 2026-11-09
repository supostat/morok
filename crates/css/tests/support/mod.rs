#![allow(dead_code)]

use std::borrow::Cow;

use morok_css::{CompiledTokens, Diagnostic, SourceFile, TokenId, compile_stylesheets, compile_tokens};
use morok_style::{
    ColorValue, Declaration, Length, LengthPercentage, LengthPercentageAuto, Pixels, Shadow, StyleSheet, Token,
    TokenTerm,
};

pub(crate) const BASE_TOKENS_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../design/system/tokens.css");
pub(crate) const STYLESHEET_PATH: &str = "component.css";

pub(crate) fn base_tokens() -> CompiledTokens {
    let source = SourceFile::read(BASE_TOKENS_FILE).expect("the base tokens file is readable");
    compile_tokens(&source).unwrap_or_else(|diagnostics| panic!("the base tokens compile:\n{diagnostics}"))
}

pub(crate) fn style_sheet(stylesheet: &str) -> StyleSheet {
    compile_stylesheets(&[SourceFile::new(STYLESHEET_PATH, stylesheet)], &base_tokens())
        .unwrap_or_else(|diagnostics| panic!("the stylesheet compiles:\n{diagnostics}"))
}

pub(crate) fn rejected(stylesheet: &str) -> Vec<Diagnostic> {
    compile_stylesheets(&[SourceFile::new(STYLESHEET_PATH, stylesheet)], &base_tokens())
        .expect_err("the stylesheet is rejected")
        .as_slice()
        .to_vec()
}

/// The declarations a rule body compiles into.
pub(crate) fn declarations(rule_body: &str) -> Vec<Declaration> {
    let style_sheet = style_sheet(&format!(".subject {{ {rule_body} }}"));
    style_sheet.rules[0].declarations.to_vec()
}

pub(crate) fn length_token(name: &str) -> Token<Pixels> {
    match base_tokens().id(name) {
        Some(TokenId::Length(token)) => token,
        other => panic!("`--{name}` is not a length token: {other:?}"),
    }
}

pub(crate) fn shadow_token(name: &str) -> Token<Shadow> {
    match base_tokens().id(name) {
        Some(TokenId::Shadow(token)) => token,
        other => panic!("`--{name}` is not a shadow token: {other:?}"),
    }
}

pub(crate) fn color(name: &str) -> ColorValue {
    match base_tokens().id(name) {
        Some(TokenId::Color(token)) => ColorValue::Token(token),
        other => panic!("`--{name}` is not a color token: {other:?}"),
    }
}

pub(crate) fn px(pixels: f32) -> LengthPercentage {
    LengthPercentage {
        pixels: Pixels(pixels),
        fraction: None,
        tokens: Cow::Borrowed(&[]),
    }
}

pub(crate) fn fraction(fraction: f32) -> LengthPercentage {
    LengthPercentage {
        fraction: Some(fraction),
        ..px(0.0)
    }
}

pub(crate) fn token(name: &str) -> LengthPercentage {
    LengthPercentage {
        tokens: Cow::Owned(vec![TokenTerm {
            token: length_token(name),
            factor: 1.0,
        }]),
        ..px(0.0)
    }
}

pub(crate) fn sized(length: LengthPercentage) -> LengthPercentageAuto {
    LengthPercentageAuto::LengthPercentage(length)
}

pub(crate) fn px_length(pixels: f32) -> Length {
    Length {
        pixels: Pixels(pixels),
        tokens: Cow::Borrowed(&[]),
    }
}

pub(crate) fn token_length(name: &str) -> Length {
    Length {
        pixels: Pixels(0.0),
        tokens: Cow::Owned(vec![TokenTerm {
            token: length_token(name),
            factor: 1.0,
        }]),
    }
}
