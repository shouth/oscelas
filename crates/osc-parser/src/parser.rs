mod common;
mod decl;
mod expr;
mod member;
mod osc_file;

use std::ops::Range;

use syntree::pointer::PointerUsize;
use syntree::{Builder as TreeBuilder, Checkpoint as TreeCheckpoint, Tree};

use crate::diagnostic::SyntaxDiagnostic;
use crate::lexer::{LexedToken, LexicalAnalyzer};
use crate::syntax::OscSyntaxKind::{self, *};
use crate::syntax::OscSyntaxKindSet;

pub use common::*;
pub use decl::*;
pub use expr::*;
pub use member::*;
pub use osc_file::*;

#[derive(Debug, Clone)]
pub struct Checkpoint(TreeCheckpoint<PointerUsize>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Indentation {
    Normal,
    Extra,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Enclosure {
    Present(OscSyntaxKind),
    Absent(OscSyntaxKind),
}

pub struct Parser<'a> {
    source: &'a str,
    lexer: LexicalAnalyzer<'a>,
    next: LexedToken,
    builder: TreeBuilder<OscSyntaxKind, u32, usize>,
    diagnostic: Vec<SyntaxDiagnostic>,
    expected: OscSyntaxKindSet,
    leading_trivia: bool,
    indentations: Vec<Indentation>,
    recovering: bool,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Parser<'a> {
        let mut lexer = LexicalAnalyzer::new(source);
        let next = lexer.next_token();

        let mut parser = Parser {
            source,
            lexer,
            next,
            builder: TreeBuilder::new(),
            diagnostic: Vec::new(),
            expected: OscSyntaxKindSet::new(),
            leading_trivia: false,
            indentations: Vec::new(),
            recovering: false,
        };
        parser.skip_trivia();
        parser
    }

    pub fn offset(&self) -> usize {
        self.lexer.offset() - self.next.length
    }

    fn skip_trivia(&mut self) {
        while let kind @ (WHITESPACE | COMMENT | TRIVIAL_NEWLINE) = self.next.kind {
            let token = self.do_bump();
            self.builder.token(kind, token.length).unwrap();
            self.leading_trivia = true;
        }
    }

    pub fn current_token_range(&mut self) -> Range<usize> {
        let start = self.offset();
        let end = start + self.next.length;
        start..end
    }

    pub fn current_token_kind(&mut self) -> OscSyntaxKind {
        self.next.kind
    }

    pub fn has_leading_trivia(&self) -> bool {
        self.leading_trivia
    }

    pub fn diagnostic(&mut self, diagnostic: SyntaxDiagnostic) {
        self.diagnostic.push(diagnostic);
    }

    pub fn unexpected(&mut self) {
        if self.next.kind == INDENT {
            let start = self.offset();
            let length = self.source[start..]
                .chars()
                .take_while(|c| [' ', '\t'].contains(c))
                .count();
            let range = start..start + length;
            self.diagnostic(SyntaxDiagnostic::UnexpectedIndentation { range });
        } else if !self.recovering {
            let range = self.current_token_range();
            let expected = self.expected;
            let found = self.current_token_kind();
            self.diagnostic(SyntaxDiagnostic::UnexpectedToken {
                range,
                expected,
                found,
            });
        }
    }

    fn do_bump(&mut self) -> LexedToken {
        let current = self.next;
        self.next = self.lexer.next_token();
        current
    }

    fn bump(&mut self, kind: OscSyntaxKind) {
        self.recovering = kind == ERROR;

        if self.next.kind == INDENT {
            let indentation = match kind {
                ERROR => Indentation::Extra,
                _ => Indentation::Normal,
            };
            self.indentations.push(indentation);
        }

        loop {
            let token = self.do_bump();
            self.builder.token(kind, token.length).unwrap();
            self.expected.clear();
            self.leading_trivia = false;
            self.skip_trivia();

            if self.next.kind != DEDENT || self.indentations.pop() == Some(Indentation::Normal) {
                break;
            }
        }
    }

    fn do_check(&mut self, kinds: OscSyntaxKindSet) -> Option<OscSyntaxKind> {
        self.expected |= kinds;
        let next = self.next.kind;

        let keyword = match next {
            IDENTIFIER => {
                let begin = self.offset();
                let length = self.next.length;
                let lexeme = &self.source[begin..][..length];
                kinds
                    .iter()
                    .find(|kind| kind.static_token() == Some(lexeme))
            }
            _ => None,
        };

        match keyword {
            Some(_) => keyword,
            None => kinds.contains(next).then(|| next),
        }
    }

    pub fn check(&mut self, kinds: impl Into<OscSyntaxKindSet>) -> bool {
        self.do_check(kinds.into()).is_some()
    }

    pub fn eat(&mut self, kinds: impl Into<OscSyntaxKindSet>) -> bool {
        let result = self.do_check(kinds.into());
        if let Some(kind) = result {
            self.bump(kind);
        }
        result.is_some()
    }

    pub fn expect(&mut self, kinds: impl Into<OscSyntaxKindSet>) -> bool {
        let result = self.eat(kinds);
        if !result {
            self.unexpected();
        }
        result
    }

    pub fn left(&mut self, left: OscSyntaxKind) -> Enclosure {
        self.lexer.left();
        if self.expect(left) {
            Enclosure::Present(left)
        } else {
            self.lexer.right();
            Enclosure::Absent(left)
        }
    }

    pub fn right(&mut self, enclosure: Enclosure, right: OscSyntaxKind) -> bool {
        let result = self.expect(right);
        let left = match enclosure {
            Enclosure::Present(left) => {
                self.lexer.right();
                left
            }
            Enclosure::Absent(left) => {
                left
            }
        };

        match (left, right) {
            (LEFT_PAREN, RIGHT_PAREN) | (LEFT_BRACKET, RIGHT_BRACKET) => (),
            _ => panic!("Bug: mismatched enclosure parser"),
        }
        result
    }

    pub fn eof(&mut self) -> bool {
        self.next.kind == EOF
    }

    pub fn error(&mut self) {
        self.bump(ERROR);
    }

    pub fn open(&mut self) -> Checkpoint {
        Checkpoint(self.builder.checkpoint().unwrap())
    }

    pub fn close(&mut self, checkpoint: Checkpoint, kind: OscSyntaxKind) {
        self.builder.close_at(&checkpoint.0, kind).unwrap();
    }

    pub fn recover(&mut self, recovery: impl Into<OscSyntaxKindSet>) {
        let recovery = recovery.into();
        while !(recovery | EOF).contains(self.next.kind) {
            self.error();
        }
    }

    pub fn finish(mut self) -> (Vec<SyntaxDiagnostic>, Tree<OscSyntaxKind, u32, usize>) {
        self.recover(EOF);
        self.bump(EOF);

        let mut diagnostics = Vec::new();
        diagnostics.extend(self.lexer.finish());
        diagnostics.extend(self.diagnostic);

        (diagnostics, self.builder.build().unwrap())
    }
}

pub fn to_osc_syntax_string(source: &str) -> String {
    let mut p = Parser::new(source);
    parse_osc_file(&mut p);
    let (_, tree) = p.finish();

    let mut writer = Vec::new();
    syntree::print::print_with_source(&mut writer, &tree, source).unwrap();
    unsafe { String::from_utf8_unchecked(writer) }
}
