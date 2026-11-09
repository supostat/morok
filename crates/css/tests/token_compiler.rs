use std::borrow::Cow;

use morok_css::{CompiledTokens, Diagnostic, SchemaPath, SourceFile, TokenId, compile_tokens, generate_tokens_source};
use morok_style::{
    Color, Easing, FontFamilies, FontWeight, Leading, Milliseconds, Pixels, Shadow, ShadowLayer, StepPosition, Theme,
    Token,
};

const BASE_TOKENS_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../design/system/tokens.css");

fn base_theme() -> CompiledTokens {
    let source = SourceFile::read(BASE_TOKENS_FILE).expect("the base tokens file is readable");
    compiled(&source)
}

fn tokens_file(light_declarations: &str, dark_declarations: &str) -> SourceFile {
    SourceFile::new(
        "tokens.css",
        format!(
            ":root {{\n{light_declarations}\n}}\n@media (prefers-color-scheme: dark) {{\n  :root {{\n{dark_declarations}\n  }}\n}}\n"
        ),
    )
}

fn same_in_both_schemes(declarations: &str) -> SourceFile {
    tokens_file(declarations, declarations)
}

fn compiled(source: &SourceFile) -> CompiledTokens {
    compile_tokens(source).unwrap_or_else(|diagnostics| panic!("the tokens file compiles:\n{diagnostics}"))
}

fn rejected(source: &SourceFile) -> Vec<Diagnostic> {
    compile_tokens(source)
        .expect_err("the tokens file is rejected")
        .as_slice()
        .to_vec()
}

fn messages(diagnostics: &[Diagnostic]) -> Vec<&str> {
    diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect()
}

fn opaque(red: u8, green: u8, blue: u8) -> Color {
    Color {
        red,
        green,
        blue,
        alpha: 1.0,
    }
}

fn table_sizes(theme: &Theme) -> [usize; 8] {
    [
        theme.colors.len(),
        theme.lengths.len(),
        theme.font_families.len(),
        theme.leadings.len(),
        theme.font_weights.len(),
        theme.shadows.len(),
        theme.durations.len(),
        theme.easings.len(),
    ]
}

fn color(tokens: &CompiledTokens, name: &str) -> Token<Color> {
    match tokens.id(name) {
        Some(TokenId::Color(token)) => token,
        other => panic!("`--{name}` is not a color token: {other:?}"),
    }
}

fn length(tokens: &CompiledTokens, name: &str) -> Token<Pixels> {
    match tokens.id(name) {
        Some(TokenId::Length(token)) => token,
        other => panic!("`--{name}` is not a length token: {other:?}"),
    }
}

fn font_families(tokens: &CompiledTokens, name: &str) -> Token<FontFamilies> {
    match tokens.id(name) {
        Some(TokenId::FontFamilies(token)) => token,
        other => panic!("`--{name}` is not a font families token: {other:?}"),
    }
}

fn leading(tokens: &CompiledTokens, name: &str) -> Token<Leading> {
    match tokens.id(name) {
        Some(TokenId::Leading(token)) => token,
        other => panic!("`--{name}` is not a leading token: {other:?}"),
    }
}

fn font_weight(tokens: &CompiledTokens, name: &str) -> Token<FontWeight> {
    match tokens.id(name) {
        Some(TokenId::FontWeight(token)) => token,
        other => panic!("`--{name}` is not a font weight token: {other:?}"),
    }
}

fn shadow(tokens: &CompiledTokens, name: &str) -> Token<Shadow> {
    match tokens.id(name) {
        Some(TokenId::Shadow(token)) => token,
        other => panic!("`--{name}` is not a shadow token: {other:?}"),
    }
}

fn duration(tokens: &CompiledTokens, name: &str) -> Token<Milliseconds> {
    match tokens.id(name) {
        Some(TokenId::Duration(token)) => token,
        other => panic!("`--{name}` is not a duration token: {other:?}"),
    }
}

fn easing(tokens: &CompiledTokens, name: &str) -> Token<Easing> {
    match tokens.id(name) {
        Some(TokenId::Easing(token)) => token,
        other => panic!("`--{name}` is not an easing token: {other:?}"),
    }
}

#[test]
fn the_base_theme_compiles_every_token_into_both_schemes() {
    let tokens = base_theme();

    assert_eq!(tokens.entries.len(), 58);
    assert_eq!(table_sizes(&tokens.light), [22, 20, 3, 2, 3, 3, 3, 2]);
    assert_eq!(table_sizes(&tokens.dark), [22, 20, 3, 2, 3, 3, 3, 2]);
}

#[test]
fn the_base_theme_colors_differ_by_scheme() {
    let tokens = base_theme();

    let background = color(&tokens, "color-bg");
    assert_eq!(*tokens.light.resolve(background), opaque(0xF3, 0xF4, 0xF6));
    assert_eq!(*tokens.dark.resolve(background), opaque(0x11, 0x13, 0x17));
    let scrim = color(&tokens, "color-scrim");
    assert_eq!(
        *tokens.light.resolve(scrim),
        Color {
            red: 17,
            green: 19,
            blue: 24,
            alpha: 0.32
        }
    );
    assert_eq!(
        *tokens.dark.resolve(scrim),
        Color {
            red: 0,
            green: 0,
            blue: 0,
            alpha: 0.5
        }
    );
}

#[test]
fn the_base_theme_geometry_and_typography_compile() {
    let tokens = base_theme();
    let light = &tokens.light;

    assert_eq!(*light.resolve(length(&tokens, "space-1")), Pixels(4.0));
    assert_eq!(*light.resolve(length(&tokens, "radius-full")), Pixels(999.0));
    assert_eq!(*light.resolve(length(&tokens, "text-md")), Pixels(13.0));
    assert_eq!(*light.resolve(leading(&tokens, "text-leading")), Leading(1.45));
    assert_eq!(*light.resolve(leading(&tokens, "text-leading-tight")), Leading(1.25));
    assert_eq!(
        *light.resolve(font_weight(&tokens, "text-weight-semibold")),
        FontWeight(600)
    );
    assert_eq!(
        *light.resolve(font_families(&tokens, "font-mono")).names,
        ["SF Mono", "ui-monospace", "JetBrains Mono", "monospace"].map(Cow::Borrowed)
    );
}

#[test]
fn the_base_theme_shadows_keep_every_layer() {
    let tokens = base_theme();
    let float = shadow(&tokens, "shadow-float");

    assert_eq!(
        *tokens.light.resolve(float).layers,
        [
            ShadowLayer {
                offset_x: Pixels(0.0),
                offset_y: Pixels(8.0),
                blur: Pixels(24.0),
                spread: Pixels(0.0),
                color: Color {
                    red: 17,
                    green: 19,
                    blue: 24,
                    alpha: 0.14
                },
                inset: false,
            },
            ShadowLayer {
                offset_x: Pixels(0.0),
                offset_y: Pixels(1.0),
                blur: Pixels(3.0),
                spread: Pixels(0.0),
                color: Color {
                    red: 17,
                    green: 19,
                    blue: 24,
                    alpha: 0.10
                },
                inset: false,
            },
        ]
    );
    assert_eq!(
        tokens.dark.resolve(float).layers[1],
        ShadowLayer {
            offset_x: Pixels(0.0),
            offset_y: Pixels(0.0),
            blur: Pixels(0.0),
            spread: Pixels(1.0),
            color: Color {
                red: 255,
                green: 255,
                blue: 255,
                alpha: 0.04
            },
            inset: false,
        }
    );
}

#[test]
fn the_base_theme_motion_compiles_into_durations_and_easings() {
    let tokens = base_theme();
    let light = &tokens.light;

    assert_eq!(*light.resolve(duration(&tokens, "motion-fast")), Milliseconds(120.0));
    assert_eq!(*light.resolve(duration(&tokens, "motion-slow")), Milliseconds(280.0));
    assert_eq!(
        *light.resolve(easing(&tokens, "motion-ease-out")),
        Easing::CubicBezier {
            x1: 0.0,
            y1: 0.0,
            x2: 0.2,
            y2: 1.0
        }
    );
}

#[test]
fn an_unknown_family_is_an_error() {
    let diagnostics = rejected(&same_in_both_schemes("  --glow-1: 4px;"));

    assert_eq!(
        messages(&diagnostics),
        ["unknown token family `glow`", "unknown token family `glow`"]
    );
    assert_eq!(
        diagnostics[0].hint,
        "token families are color-, space-, radius-, font-, text-, shadow-, motion-"
    );
}

#[test]
fn a_value_kind_the_family_does_not_admit_is_an_error() {
    let rejections = [
        ("--space-1: #fff;", "`--space-1` expects a length in px"),
        ("--color-bg: 4px;", "`--color-bg` expects a color"),
        ("--radius-sm: 1.5;", "`--radius-sm` expects a length in px"),
        ("--font-ui: 12px;", "`--font-ui` expects a list of font families"),
        (
            "--shadow-raised: 4px;",
            "`--shadow-raised` expects a list of shadow layers",
        ),
        ("--motion-fast: 4px;", "`--motion-fast` expects a duration or an easing"),
    ];
    for (declaration, message) in rejections {
        let diagnostics = rejected(&same_in_both_schemes(declaration));

        assert_eq!(messages(&diagnostics), [message, message], "{declaration}");
    }
}

#[test]
fn a_name_present_in_one_scheme_only_is_an_error() {
    let missing_from_dark = rejected(&tokens_file(
        "  --color-bg: #fff;\n  --color-ink: #000;",
        "  --color-bg: #000;",
    ));
    let missing_from_light = rejected(&tokens_file(
        "  --color-bg: #fff;",
        "  --color-bg: #000;\n  --color-ink: #fff;",
    ));

    assert_eq!(
        messages(&missing_from_dark),
        ["`--color-ink` is defined for the light scheme only"]
    );
    assert_eq!(
        messages(&missing_from_light),
        ["`--color-ink` is defined for the dark scheme only"]
    );
}

#[test]
fn a_name_declared_twice_in_one_scheme_is_an_error() {
    let diagnostics = rejected(&tokens_file(
        "  --color-bg: #fff;\n  --color-bg: #eee;",
        "  --color-bg: #000;",
    ));

    assert_eq!(
        messages(&diagnostics),
        ["`--color-bg` is declared twice in the light scheme"]
    );
    assert_eq!(diagnostics[0].hint, "its first declaration is on line 2");
    assert_eq!((diagnostics[0].line, diagnostics[0].column), (3, 3));
}

#[test]
fn a_text_token_is_a_leading_a_weight_or_a_size_by_its_name() {
    let tokens = compiled(&same_in_both_schemes(
        "  --text-leading-tight: 1.25;\n  --text-weight-bold: 700;\n  --text-md: 13px;",
    ));

    assert_eq!(
        *tokens.light.resolve(leading(&tokens, "text-leading-tight")),
        Leading(1.25)
    );
    assert_eq!(
        *tokens.light.resolve(font_weight(&tokens, "text-weight-bold")),
        FontWeight(700)
    );
    assert_eq!(*tokens.light.resolve(length(&tokens, "text-md")), Pixels(13.0));
}

#[test]
fn a_text_token_rejects_the_value_of_another_text_kind() {
    let rejections = [
        (
            "--text-leading: 13px;",
            "`--text-leading` expects a unitless line height",
        ),
        (
            "--text-weight-bold: 1.5;",
            "`--text-weight-bold` expects an integer font weight from 1 to 1000",
        ),
        ("--text-md: 1.45;", "`--text-md` expects a length in px"),
    ];
    for (declaration, message) in rejections {
        let diagnostics = rejected(&same_in_both_schemes(declaration));

        assert_eq!(messages(&diagnostics), [message, message], "{declaration}");
    }
}

#[test]
fn a_motion_value_is_a_duration_or_an_easing_by_its_syntax() {
    let tokens = compiled(&same_in_both_schemes(
        "  --motion-fast: 120ms;\n  --motion-slow: 0.28s;\n  --motion-ease: cubic-bezier(0.2, 0, 0, 1);\n  --motion-tick: steps(4, jump-none);\n  --motion-even: linear;",
    ));
    let light = &tokens.light;

    assert_eq!(*light.resolve(duration(&tokens, "motion-fast")), Milliseconds(120.0));
    assert_eq!(*light.resolve(duration(&tokens, "motion-slow")), Milliseconds(280.0));
    assert_eq!(
        *light.resolve(easing(&tokens, "motion-ease")),
        Easing::CubicBezier {
            x1: 0.2,
            y1: 0.0,
            x2: 0.0,
            y2: 1.0
        }
    );
    assert_eq!(
        *light.resolve(easing(&tokens, "motion-tick")),
        Easing::Steps {
            count: 4,
            position: StepPosition::JumpNone
        }
    );
    assert_eq!(*light.resolve(easing(&tokens, "motion-even")), Easing::Linear);
}

#[test]
fn a_number_too_large_to_represent_is_rejected() {
    let rejections = [
        (
            "--motion-ease: cubic-bezier(0, 1e999, 0, 1);",
            "`--motion-ease` expects a duration or an easing",
        ),
        (
            "--motion-fast: 1e999ms;",
            "`--motion-fast` expects a duration or an easing",
        ),
        ("--space-1: 1e999px;", "`--space-1` expects a length in px"),
        (
            "--text-leading: 1e999;",
            "`--text-leading` expects a unitless line height",
        ),
    ];
    for (declaration, message) in rejections {
        let diagnostics = rejected(&same_in_both_schemes(declaration));

        assert_eq!(messages(&diagnostics), [message, message], "{declaration}");
    }
}

#[test]
fn a_motion_token_holds_one_kind_of_value_in_both_schemes() {
    let diagnostics = rejected(&tokens_file("  --motion-fast: 120ms;", "  --motion-fast: linear;"));

    assert_eq!(
        messages(&diagnostics),
        ["`--motion-fast` is a duration in the light scheme and an easing in the dark scheme"]
    );
}

#[test]
fn a_shadow_keeps_its_layers_and_inset() {
    let tokens = compiled(&same_in_both_schemes(
        "  --shadow-well: inset 0 1px 2px rgba(0, 0, 0, 0.2), 0 8px 24px -4px #11131780;",
    ));

    assert_eq!(
        *tokens.light.resolve(shadow(&tokens, "shadow-well")).layers,
        [
            ShadowLayer {
                offset_x: Pixels(0.0),
                offset_y: Pixels(1.0),
                blur: Pixels(2.0),
                spread: Pixels(0.0),
                color: Color {
                    red: 0,
                    green: 0,
                    blue: 0,
                    alpha: 0.2
                },
                inset: true,
            },
            ShadowLayer {
                offset_x: Pixels(0.0),
                offset_y: Pixels(8.0),
                blur: Pixels(24.0),
                spread: Pixels(-4.0),
                color: Color {
                    red: 0x11,
                    green: 0x13,
                    blue: 0x17,
                    alpha: f32::from(0x80_u8) / 255.0
                },
                inset: false,
            },
        ]
    );
}

#[test]
fn a_block_other_than_the_two_root_blocks_is_an_error() {
    let source = SourceFile::new(
        "tokens.css",
        ":root { --color-bg: #fff; }\n.card { --color-bg: #eee; }\n@media (prefers-color-scheme: dark) { :root { --color-bg: #000; } }\n",
    );

    let diagnostics = rejected(&source);

    assert_eq!(
        messages(&diagnostics),
        ["a tokens file holds only `:root` and `@media (prefers-color-scheme: dark)` blocks"]
    );
    assert_eq!((diagnostics[0].line, diagnostics[0].column), (2, 1));
}

#[test]
fn a_scheme_block_is_declared_once() {
    let source = SourceFile::new(
        "tokens.css",
        ":root { --color-bg: #fff; }\n:root { --color-ink: #000; }\n@media (prefers-color-scheme: dark) { :root { --color-bg: #000; } }\n",
    );

    assert_eq!(messages(&rejected(&source)), ["the light scheme is declared twice"]);
}

#[test]
fn a_file_without_a_dark_block_is_an_error() {
    let source = SourceFile::new("tokens.css", ":root { --color-bg: #fff; }\n");

    assert_eq!(messages(&rejected(&source)), ["the tokens file has no dark scheme"]);
}

#[test]
fn a_tokens_block_holds_well_formed_custom_properties_only() {
    let rejections = [
        ("color-scheme: light;", "a tokens block holds custom properties only"),
        ("--Color-BG: #fff;", "`--Color-BG` is not lowercase kebab-case"),
        ("--color: #fff;", "`--color` has a family but no name"),
        ("--color-bg;", "`--color-bg` has no value"),
    ];
    for (declaration, message) in rejections {
        let diagnostics = rejected(&same_in_both_schemes(declaration));

        assert_eq!(messages(&diagnostics), [message, message], "{declaration}");
    }
}

#[test]
fn a_diagnostic_carries_file_line_and_column() {
    let source = tokens_file(
        "  --color-bg: #fff;\n  --space-1: #fff;",
        "  --color-bg: #000;\n  --space-1: 4px;",
    );

    let diagnostics = rejected(&source);

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].path.to_str(), Some("tokens.css"));
    assert_eq!((diagnostics[0].line, diagnostics[0].column), (3, 14));
    assert_eq!(
        diagnostics[0].to_string(),
        "error[morok-css]: `--space-1` expects a length in px\n --> tokens.css:3:14\n  |\n3 |   --space-1: #fff;\n  |              ^^^^ for example `4px`"
    );
}

#[test]
fn diagnostics_are_ordered_by_position_and_align_a_wide_line_number() {
    let padding = "\n".repeat(8);
    let source = tokens_file(
        &format!("  --color-ink: #000;{padding}  --glow-1: 4px;"),
        "  --color-bg: #000;",
    );

    let diagnostics = compile_tokens(&source).expect_err("the tokens file is rejected");

    assert_eq!(
        messages(diagnostics.as_slice()),
        [
            "`--color-ink` is defined for the light scheme only",
            "unknown token family `glow`",
            "`--color-bg` is defined for the dark scheme only",
        ]
    );
    assert_eq!(
        diagnostics.as_slice()[1].to_string(),
        "error[morok-css]: unknown token family `glow`\n  --> tokens.css:10:3\n   |\n10 |   --glow-1: 4px;\n   |   ^^^^^^^^ token families are color-, space-, radius-, font-, text-, shadow-, motion-"
    );
}

#[test]
fn two_compilations_of_one_file_generate_identical_source() {
    let first = generate_tokens_source(&base_theme(), SchemaPath::Crate);
    let second = generate_tokens_source(&base_theme(), SchemaPath::Crate);

    assert_eq!(first, second);
}
