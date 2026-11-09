mod support;

use morok_css::{SourceFile, TokenId, compile_stylesheets};
use morok_style::{BoxShadow, Color, ColorValue, Corner, Declaration, Pixels, Side};
use support::{STYLESHEET_PATH, base_tokens, declarations, rejected, sized, token, token_length};

#[test]
fn tokens_of_the_families_declarations_take_resolve_to_the_ids_of_the_base_theme() {
    let tokens = base_tokens();
    let Some(TokenId::Color(text_color)) = tokens.id("color-text") else {
        panic!("`--color-text` is a color token");
    };
    let Some(TokenId::Shadow(float_shadow)) = tokens.id("shadow-float") else {
        panic!("`--shadow-float` is a shadow token");
    };

    let compiled = declarations(
        "color: var(--color-text); padding-left: var(--space-4); border-top-left-radius: var(--radius-md); \
         height: var(--text-2xl); box-shadow: var(--shadow-float);",
    );

    assert_eq!(
        compiled,
        [
            Declaration::Color(ColorValue::Token(text_color)),
            Declaration::Padding(Side::Left, token("space-4")),
            Declaration::BorderRadius(Corner::TopLeft, token_length("radius-md")),
            Declaration::Height(sized(token("text-2xl"))),
            Declaration::BoxShadow(BoxShadow::Token(float_shadow)),
        ]
    );
    assert_eq!(
        *tokens.light.resolve(text_color),
        Color {
            red: 0x15,
            green: 0x17,
            blue: 0x1C,
            alpha: 1.0
        }
    );
    assert_eq!(
        *tokens.dark.resolve(text_color),
        Color {
            red: 0xE8,
            green: 0xEB,
            blue: 0xF0,
            alpha: 1.0
        }
    );
    assert_eq!(*tokens.light.resolve(token("space-4").tokens[0].token), Pixels(16.0));
    assert_eq!(
        *tokens.light.resolve(token_length("radius-md").tokens[0].token),
        Pixels(8.0)
    );
    assert_eq!(*tokens.light.resolve(token("text-2xl").tokens[0].token), Pixels(28.0));
    assert_eq!(tokens.light.resolve(float_shadow).layers.len(), 2);
}

#[test]
fn tokens_of_the_kinds_no_property_takes_yet_are_found_and_rejected_by_kind() {
    let rejections = [
        (
            "gap: var(--font-ui);",
            "`--font-ui` is a list of font families, not a length",
        ),
        (
            "gap: var(--text-leading);",
            "`--text-leading` is a line height, not a length",
        ),
        (
            "gap: var(--text-weight-medium);",
            "`--text-weight-medium` is a font weight, not a length",
        ),
        (
            "color: var(--motion-fast);",
            "`--motion-fast` is a duration, not a color",
        ),
        (
            "color: var(--motion-ease);",
            "`--motion-ease` is an easing, not a color",
        ),
    ];
    for (declaration, message) in rejections {
        let diagnostics = rejected(&format!(".subject {{ {declaration} }}"));

        let messages: Vec<_> = diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.as_str())
            .collect();
        assert_eq!(messages, [message], "{declaration}");
    }
}

#[test]
fn every_token_of_the_base_theme_is_known_to_the_stylesheet_compiler() {
    let tokens = base_tokens();
    for entry in &tokens.entries {
        let property = match entry.id {
            TokenId::Color(_) => "color",
            TokenId::Shadow(_) => "box-shadow",
            _ => "width",
        };
        let stylesheet = SourceFile::new(
            STYLESHEET_PATH,
            format!(".subject {{ {property}: var(--{}); }}", entry.name),
        );

        let compiled = compile_stylesheets(&[stylesheet], &tokens);

        match entry.id {
            TokenId::Color(token) => assert_eq!(
                *compiled.expect("a color token compiles").rules[0].declarations,
                [Declaration::Color(ColorValue::Token(token))],
                "--{}",
                entry.name
            ),
            TokenId::Shadow(token) => assert_eq!(
                *compiled.expect("a shadow token compiles").rules[0].declarations,
                [Declaration::BoxShadow(BoxShadow::Token(token))],
                "--{}",
                entry.name
            ),
            TokenId::Length(_) => assert_eq!(
                *compiled.expect("a length token compiles").rules[0].declarations,
                [Declaration::Width(sized(token(&entry.name)))],
                "--{}",
                entry.name
            ),
            _ => {
                let diagnostics = compiled.expect_err("a token of another kind is not a length");
                let message = &diagnostics.as_slice()[0].message;
                assert!(
                    message.starts_with(&format!("`--{}` is ", entry.name)) && message.ends_with(", not a length"),
                    "{message}"
                );
            }
        }
    }
}
