use morok_css::{
    CompiledTokens, Diagnostic, SourceFile, TokenId, compile_alternate_tokens, compile_application_tokens,
    compile_tokens,
};
use morok_style::{Color, Token};

const KIT_DECLARATIONS: &str = "  --color-bg: #ffffff;\n  --space-1: 4px;\n  --color-text: #000000;";

fn tokens_file(path: &str, light_declarations: &str, dark_declarations: &str) -> SourceFile {
    SourceFile::new(
        path,
        format!(
            ":root {{\n{light_declarations}\n}}\n@media (prefers-color-scheme: dark) {{\n  :root {{\n{dark_declarations}\n  }}\n}}\n"
        ),
    )
}

fn kit() -> CompiledTokens {
    compile_tokens(&tokens_file("kit.css", KIT_DECLARATIONS, KIT_DECLARATIONS))
        .unwrap_or_else(|diagnostics| panic!("the tokens of the kit compile:\n{diagnostics}"))
}

fn application(declarations: &str) -> SourceFile {
    tokens_file("application.css", declarations, declarations)
}

fn compiled_application(declarations: &str) -> CompiledTokens {
    compile_application_tokens(&kit(), &application(declarations))
        .unwrap_or_else(|diagnostics| panic!("the tokens of the application compile:\n{diagnostics}"))
}

fn color(tokens: &CompiledTokens, name: &str) -> Token<Color> {
    match tokens.id(name) {
        Some(TokenId::Color(token)) => token,
        other => panic!("`--{name}` is not a color token: {other:?}"),
    }
}

fn opaque(red: u8, green: u8, blue: u8) -> Color {
    Color {
        red,
        green,
        blue,
        alpha: 1.0,
    }
}

fn diagnostics(compiled: Result<CompiledTokens, morok_css::Diagnostics>) -> Vec<Diagnostic> {
    compiled.expect_err("the tokens file is rejected").as_slice().to_vec()
}

#[test]
fn a_token_of_the_kit_keeps_its_id_and_takes_the_value_of_the_application() {
    let kit = kit();
    let tokens = compiled_application("  --color-text: #101010;\n  --space-1: 6px;\n  --color-bg: #fafafa;");

    for name in ["color-bg", "space-1", "color-text"] {
        assert_eq!(tokens.id(name), kit.id(name), "`--{name}` keeps the ID of the kit");
    }
    assert_eq!(*tokens.light.resolve(color(&tokens, "color-text")), opaque(16, 16, 16));
    assert_eq!(*tokens.light.resolve(color(&tokens, "color-bg")), opaque(250, 250, 250));
}

#[test]
fn a_token_of_the_application_gets_an_id_after_those_of_the_kit() {
    let kit = kit();
    let tokens = compiled_application(
        "  --color-chart-1: #148f78;\n  --color-bg: #ffffff;\n  --space-1: 4px;\n  --color-text: #000000;\n  --color-chart-2: #2fb89c;",
    );

    assert_eq!(color(&tokens, "color-chart-1").index(), kit.light.colors.len());
    assert_eq!(color(&tokens, "color-chart-2").index(), kit.light.colors.len() + 1);
    assert_eq!(
        *tokens.dark.resolve(color(&tokens, "color-chart-2")),
        opaque(47, 184, 156)
    );
}

#[test]
fn a_token_of_the_kit_the_application_leaves_out_is_an_error_at_its_light_block() {
    let diagnostics = diagnostics(compile_application_tokens(
        &kit(),
        &application("  --color-bg: #ffffff;\n  --color-text: #000000;"),
    ));

    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(diagnostics[0].message, "`--space-1` is not defined");
    assert_eq!(
        diagnostics[0].to_string(),
        "error[morok-css]: `--space-1` is not defined\n --> application.css:1:1\n  |\n1 | :root {\n  | ^^^^^ the tokens file of an application defines every token of the kit"
    );
}

#[test]
fn a_file_with_rejected_values_is_not_held_against_the_kit() {
    let diagnostics = diagnostics(compile_application_tokens(
        &kit(),
        &application("  --color-bg: 4px;\n  --space-1: 4px;\n  --color-text: #000000;"),
    ));

    let messages: Vec<&str> = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    assert_eq!(
        messages,
        ["`--color-bg` expects a color", "`--color-bg` expects a color"]
    );
}

#[test]
fn an_alternate_tokens_file_keeps_the_ids_of_the_first_and_carries_its_own_values() {
    let first = compiled_application(
        "  --color-bg: #ffffff;\n  --space-1: 4px;\n  --color-text: #000000;\n  --color-chart-1: #148f78;",
    );
    let alternate = compile_alternate_tokens(
        &first,
        &tokens_file(
            "ember.css",
            "  --color-chart-1: #c2410c;\n  --color-text: #1c1917;\n  --space-1: 4px;\n  --color-bg: #fffbf5;",
            "  --color-chart-1: #fb923c;\n  --color-text: #fafaf9;\n  --space-1: 4px;\n  --color-bg: #1c1917;",
        ),
    )
    .unwrap_or_else(|diagnostics| panic!("the alternate tokens compile:\n{diagnostics}"));

    assert_eq!(alternate.entries, first.entries);
    assert_eq!(
        *alternate.light.resolve(color(&first, "color-chart-1")),
        opaque(194, 65, 12)
    );
    assert_eq!(*alternate.dark.resolve(color(&first, "color-bg")), opaque(28, 25, 23));
}

#[test]
fn an_alternate_tokens_file_defines_every_token_of_the_first_and_no_other() {
    let first = compiled_application(
        "  --color-bg: #ffffff;\n  --space-1: 4px;\n  --color-text: #000000;\n  --color-chart-1: #148f78;",
    );
    let declarations = "  --color-bg: #ffffff;\n  --space-1: 4px;\n  --color-text: #000000;\n  --color-glow: #ff0000;";

    let diagnostics = diagnostics(compile_alternate_tokens(
        &first,
        &tokens_file("ember.css", declarations, declarations),
    ));

    let placed: Vec<(&str, usize, usize)> = diagnostics
        .iter()
        .map(|diagnostic| (diagnostic.message.as_str(), diagnostic.line, diagnostic.column))
        .collect();
    assert_eq!(
        placed,
        [
            ("`--color-chart-1` is not defined", 1, 1),
            ("`--color-glow` is not a token of the first tokens file", 5, 3),
        ]
    );
}
