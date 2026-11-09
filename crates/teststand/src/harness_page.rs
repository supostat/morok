use crate::markup::TEXT_ATTRIBUTE;

/// What every surface of the kit starts from: border boxes without margin or padding, the
/// font of the system interface, grayscale anti-aliasing of text, and a root that is the whole
/// of the surface unless a rule sizes it.
const RESET: &str = "*, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }\n\
                     html { font-family: system-ui; -webkit-font-smoothing: antialiased; }\n\
                     html, body, body > div { height: 100%; }";

/// A text of the kit is the inline content of its parent: `text-overflow` on the parent cuts
/// it. On the page the text has a `div` of its own, which the property of the parent does not
/// reach, so the page hands it down.
const TEXT_OVERFLOW_SCRIPT: &str = "for (const text of document.querySelectorAll('[TEXT_ATTRIBUTE]')) {\n\
    const parent = getComputedStyle(text.parentElement);\n\
    if (parent.textOverflow === 'ellipsis' && parent.overflowX !== 'visible') {\n\
    text.style.overflowX = 'clip';\n\
    text.style.textOverflow = 'ellipsis';\n\
    }\n\
    }";

/// The page Chrome renders a case from: the reset, the tokens as they are, the stylesheets
/// of the suite and the markup of the tree.
pub(crate) fn document(tokens: &str, stylesheets: &[String], markup: &str) -> String {
    let mut document = String::from("<!DOCTYPE html>\n<html>\n<head>\n<meta charset=\"utf-8\">\n");
    for style in [RESET, tokens]
        .into_iter()
        .chain(stylesheets.iter().map(String::as_str))
    {
        document.push_str("<style>\n");
        document.push_str(style);
        document.push_str("\n</style>\n");
    }
    document.push_str("</head>\n<body>");
    document.push_str(markup);
    document.push_str("<script>\n");
    document.push_str(&TEXT_OVERFLOW_SCRIPT.replace("TEXT_ATTRIBUTE", TEXT_ATTRIBUTE));
    document.push_str("\n</script></body>\n</html>\n");
    document
}
