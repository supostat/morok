use crate::comment::Comment;
use crate::source_language::SourceLanguage;

pub(crate) fn comments(source: &str, language: SourceLanguage) -> Vec<Comment> {
    let mut scanner = Scanner {
        characters: source.chars().collect(),
        position: 0,
        line: 1,
        language,
        comments: Vec::new(),
    };
    scanner.scan();
    scanner.comments
}

struct Scanner {
    characters: Vec<char>,
    position: usize,
    line: usize,
    language: SourceLanguage,
    comments: Vec<Comment>,
}

impl Scanner {
    fn scan(&mut self) {
        while let Some(current) = self.peek(0) {
            match current {
                '/' if self.peek(1) == Some('/') && self.language.has_line_comments() => self.line_comment(),
                '/' if self.peek(1) == Some('*') => self.block_comment(),
                '"' => self.quoted_string('"'),
                '\'' if self.language.quotes_strings_with_apostrophes() => self.quoted_string('\''),
                '\'' => self.character_literal_or_lifetime(),
                'r' if self.language.has_raw_strings() && self.starts_raw_string() => self.raw_string(),
                _ => self.advance(),
            }
        }
    }

    fn peek(&self, offset: usize) -> Option<char> {
        self.characters.get(self.position + offset).copied()
    }

    fn advance(&mut self) {
        if self.peek(0) == Some('\n') {
            self.line += 1;
        }
        self.position += 1;
    }

    fn line_comment(&mut self) {
        let line = self.line;
        let mut text = String::new();
        while let Some(current) = self.peek(0) {
            if current == '\n' {
                break;
            }
            text.push(current);
            self.advance();
        }
        self.comments.push(Comment { line, text });
    }

    fn block_comment(&mut self) {
        let line = self.line;
        let mut text = String::new();
        let mut depth = 0_usize;
        while let Some(current) = self.peek(0) {
            let opens_nested_comment = current == '/' && self.peek(1) == Some('*');
            if opens_nested_comment && (depth == 0 || self.language.nests_block_comments()) {
                depth += 1;
                text.push_str("/*");
                self.advance();
                self.advance();
            } else if current == '*' && self.peek(1) == Some('/') {
                depth -= 1;
                text.push_str("*/");
                self.advance();
                self.advance();
                if depth == 0 {
                    break;
                }
            } else {
                text.push(current);
                self.advance();
            }
        }
        self.comments.push(Comment { line, text });
    }

    fn quoted_string(&mut self, quote: char) {
        self.advance();
        while let Some(current) = self.peek(0) {
            if current == '\\' {
                self.advance();
                self.advance();
            } else if current == quote {
                self.advance();
                return;
            } else {
                self.advance();
            }
        }
    }

    fn starts_raw_string(&self) -> bool {
        if !self.raw_prefix_stands_alone() {
            return false;
        }
        let mut offset = 1;
        while self.peek(offset) == Some('#') {
            offset += 1;
        }
        self.peek(offset) == Some('"')
    }

    /// Whether the `r` under the cursor starts a token (`r"…"`, `br"…"`, `cr"…"`) instead of ending an identifier.
    fn raw_prefix_stands_alone(&self) -> bool {
        match self.character_before_cursor(1) {
            Some('b' | 'c') => !self.character_before_cursor(2).is_some_and(is_identifier_character),
            Some(previous) => !is_identifier_character(previous),
            None => true,
        }
    }

    fn character_before_cursor(&self, distance: usize) -> Option<char> {
        self.position
            .checked_sub(distance)
            .and_then(|index| self.characters.get(index))
            .copied()
    }

    fn raw_string(&mut self) {
        self.advance();
        let mut hashes = 0_usize;
        while self.peek(0) == Some('#') {
            hashes += 1;
            self.advance();
        }
        self.advance();
        while self.peek(0).is_some() {
            if self.peek(0) == Some('"') && (1..=hashes).all(|offset| self.peek(offset) == Some('#')) {
                for _ in 0..=hashes {
                    self.advance();
                }
                return;
            }
            self.advance();
        }
    }

    fn character_literal_or_lifetime(&mut self) {
        if self.peek(1) == Some('\\') {
            self.advance();
            self.advance();
            self.advance();
            while let Some(current) = self.peek(0) {
                self.advance();
                if current == '\'' {
                    return;
                }
            }
        } else if self.peek(2) == Some('\'') {
            self.advance();
            self.advance();
            self.advance();
        } else {
            self.advance();
        }
    }
}

fn is_identifier_character(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}
