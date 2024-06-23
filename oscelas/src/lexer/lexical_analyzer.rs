use crate::diagnostic::SyntaxDiagnostic;
use crate::syntax::OscSyntaxKind::{self, *};

use super::lookahead::{Lookahead, LookaheadSource};
use super::LexedToken;
use super::lexer::Lexer;

pub struct LexicalAnalyzer<'a> {
    source: &'a str,
    inner: Lookahead<Lexer<'a>>,
    indents: Vec<(&'a str, usize)>,
    enclosures: Vec<OscSyntaxKind>,
    is_new_line: bool,
    is_empty_line: bool,
    has_indetation_error: bool,
    diagnostics: Vec<SyntaxDiagnostic>,
}

impl<'a> LexicalAnalyzer<'a> {
    pub fn new(source: &'a str) -> LexicalAnalyzer<'a> {
        LexicalAnalyzer {
            source,
            inner: Lookahead::new(Lexer::new(source)),
            indents: vec![("", 0)],
            enclosures: Vec::new(),
            is_new_line: true,
            is_empty_line: false,
            has_indetation_error: false,
            diagnostics: Vec::new(),
        }
    }

    pub fn offset(&self) -> usize {
        self.inner.offset()
    }

    fn alter(&mut self, kind: OscSyntaxKind) -> LexedToken {
        let token = self.inner.bump();
        LexedToken { kind, ..token }
    }

    fn spawn(&self, kind: OscSyntaxKind) -> LexedToken {
        LexedToken { kind, length: 0 }
    }

    pub fn next_token(&mut self) -> LexedToken {
        if self.is_new_line {
            self.is_new_line = false;

            let (indent, indent_width) = match self.inner.nth(0).kind {
                TRIVIAL_NEWLINE => {
                    self.is_empty_line = true;
                    return self.next_token();
                }
                WHITESPACE => {
                    if self.inner.nth(1).kind == TRIVIAL_NEWLINE {
                        self.is_empty_line = true;
                        return self.inner.bump();
                    }

                    let whitespace = &self.source[self.offset()..][..self.inner.nth(0).length];
                    let indent_text_width = whitespace
                        .chars()
                        .take_while(|c| c == &' ' || c == &'\t')
                        .count();
                    let indent = &whitespace[..indent_text_width];
                    let indent_width = indent.chars().map(|c| if c == '\t' { 8 } else { 1 }).sum();
                    (indent, indent_width)
                }
                _ => ("", 0),
            };

            let (last_indent, last_indent_width) = self
                .indents
                .last()
                .expect("indentation stack is empty")
                .to_owned();

            if indent_width > last_indent_width {
                if self.has_indetation_error {
                    self.inner.bump()
                } else {
                    self.indents.push((indent, indent_width));
                    self.spawn(INDENT)
                }
            } else if indent_width < last_indent_width {
                self.indents.pop();
                let (_, last_indent_width) = self
                    .indents
                    .last()
                    .expect("indentation stack is empty")
                    .to_owned();

                if indent_width <= last_indent_width {
                    self.is_new_line = true;
                    self.has_indetation_error = false;
                    self.spawn(DEDENT)
                } else {
                    let start = self.offset();
                    let end = start + self.inner.nth(0).length;
                    self.diagnostics.push(SyntaxDiagnostic::IrregularIndentationSize {
                        range: start..end,
                        expected: last_indent_width,
                        actual: indent_width,
                    });
                    self.has_indetation_error = true;
                    self.spawn(ERROR)
                }
            } else if indent == last_indent {
                self.inner.bump()
            } else {
                let start = self.offset();
                let end = start + self.inner.nth(0).length;
                self.diagnostics.push(SyntaxDiagnostic::IrregularIndentationSequence {
                    range: start..end,
                    expected: last_indent.to_string(),
                    actual: indent.to_string(),
                });
                self.spawn(ERROR)
            }
        } else {
            match self.inner.nth(0).kind {
                TRIVIAL_NEWLINE => {
                    self.is_new_line = true;
                    if !self.is_empty_line && self.enclosures.is_empty() {
                        self.alter(NEWLINE)
                    } else {
                        self.is_empty_line = false;
                        self.inner.bump()
                    }
                }
                left @ (LEFT_BRACKET | LEFT_PAREN) => {
                    self.enclosures.push(left);
                    self.inner.bump()
                }
                right @ (RIGHT_BRACKET | RIGHT_PAREN) => {
                    match self.enclosures.last().map(|left| (left, right)) {
                        Some((LEFT_BRACKET, RIGHT_BRACKET) | (LEFT_PAREN, RIGHT_PAREN)) => {
                            self.enclosures.pop();
                        }
                        _ => {
                            // enclosures are mismatched.
                            // clear enclosures to make suceeding TRIVIAL_NEWLINE to be treated as NEWLINE.
                            // making error from mismatched enclosures is parser's responsibility.
                            self.enclosures.clear();
                        }
                    }
                    self.inner.bump()
                }
                _ => self.inner.bump(),
            }
        }
    }

    pub fn finish(self) -> Vec<SyntaxDiagnostic> {
        let mut diagnostics = self.diagnostics;
        diagnostics.extend(self.inner.finish().finish());
        diagnostics
    }
}

impl LookaheadSource for LexicalAnalyzer<'_> {
    fn next_token(&mut self) -> LexedToken {
        self.next_token()
    }
}
