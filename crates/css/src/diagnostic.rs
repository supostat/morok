use std::fmt;
use std::path::PathBuf;

use crate::source_file::SourceFile;
use crate::span::Span;

/// A compile error that names the file, line and column of the rejected construct.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub message: String,
    pub hint: String,
    pub path: PathBuf,
    pub line: usize,
    pub column: usize,
    source_line: String,
    underlined_characters: usize,
}

impl Diagnostic {
    pub(crate) fn new(source: &SourceFile, span: Span, message: impl Into<String>, hint: impl Into<String>) -> Self {
        let text = source.text();
        let line_start = text[..span.start].rfind('\n').map_or(0, |newline| newline + 1);
        let line_end = text[span.start..]
            .find('\n')
            .map_or(text.len(), |newline| span.start + newline);
        let underlined = &text[span.start..span.end.clamp(span.start, line_end)];
        Self {
            message: message.into(),
            hint: hint.into(),
            path: source.path().to_path_buf(),
            line: source.line_number(span.start),
            column: text[line_start..span.start].chars().count() + 1,
            source_line: text[line_start..line_end].trim_end_matches('\r').to_owned(),
            underlined_characters: underlined.chars().count().max(1),
        }
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let line_number = self.line.to_string();
        let gutter = " ".repeat(line_number.len());
        let indent: String = self
            .source_line
            .chars()
            .take(self.column - 1)
            .map(|character| if character == '\t' { '\t' } else { ' ' })
            .collect();
        let underline = "^".repeat(self.underlined_characters);
        writeln!(formatter, "error[morok-css]: {}", self.message)?;
        writeln!(
            formatter,
            "{gutter}--> {}:{}:{}",
            self.path.display(),
            self.line,
            self.column
        )?;
        writeln!(formatter, "{gutter} |")?;
        writeln!(formatter, "{line_number} | {}", self.source_line)?;
        write!(formatter, "{gutter} | {indent}{underline} {}", self.hint)
    }
}
