use std::borrow::Cow;

use cssparser::Parser;
use morok_style::FontFamilies;

use super::Parsed;

pub(crate) fn parse(input: &mut Parser<'_>) -> Parsed<FontFamilies> {
    let names = input.parse_comma_separated(family)?;
    Ok(FontFamilies {
        names: Cow::Owned(names),
    })
}

fn family(input: &mut Parser<'_>) -> Parsed<Cow<'static, str>> {
    if let Ok(quoted) = input.try_parse(Parser::expect_string_cloned) {
        return Ok(Cow::Owned(quoted.as_ref().to_owned()));
    }
    let mut words = vec![input.expect_ident_cloned()?.as_ref().to_owned()];
    while let Ok(word) = input.try_parse(Parser::expect_ident_cloned) {
        words.push(word.as_ref().to_owned());
    }
    Ok(Cow::Owned(words.join(" ")))
}
