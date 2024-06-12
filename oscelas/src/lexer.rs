use std::{collections::VecDeque, str::Chars};

use crate::{
    chars::{is_id_char, is_id_start_char},
    syntax::OscDslSyntaxKind::{self, *},
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct LexedToken {
    pub kind: OscDslSyntaxKind,
    pub length: usize,
}

struct Cursor<'a> {
    source: Chars<'a>,
    offset: usize,
}

impl<'a> Cursor<'_> {
    fn new(src: &str) -> Cursor {
        Cursor {
            source: src.chars(),
            offset: 0,
        }
    }

    fn first(&self) -> Option<char> {
        self.source.clone().nth(self.offset + 0)
    }

    fn second(&self) -> Option<char> {
        self.source.clone().nth(self.offset + 1)
    }

    fn third(&self) -> Option<char> {
        self.source.clone().nth(self.offset + 2)
    }

    fn bump(&mut self) -> Option<char> {
        let offset = self.offset;
        self.offset += 1;
        self.source.clone().nth(offset)
    }

    fn eat(&mut self, c: char) -> bool {
        if self.first().is_some_and(|x| x == c) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn token(&mut self, kind: OscDslSyntaxKind) -> LexedToken {
        let length = self.offset;
        self.offset = 0;
        if length > 0 {
            self.source.nth(length - 1);
        }
        LexedToken { kind, length }
    }
}

fn eat_digits(cursor: &mut Cursor, radix: u32) -> bool {
    if cursor.first().is_some_and(|c| c.is_digit(radix)) {
        while cursor.first().is_some_and(|c| c.is_digit(radix)) {
            cursor.bump();
        }
        true
    } else {
        false
    }
}

fn eat_exponent(cursor: &mut Cursor) -> bool {
    if cursor.first().is_some_and(|c| c != 'e' && c != 'E') {
        false
    } else if cursor.second().is_some_and(|c| c == '+' || c == '-')
        && cursor.third().is_some_and(|c| c.is_digit(10))
    {
        // start with 'e+', 'e-', 'E+', 'E-'
        cursor.bump();
        cursor.bump();
        eat_digits(cursor, 10);
        true
    } else if cursor.second().is_some_and(|c| c.is_digit(10)) {
        // start with 'e', 'E'
        cursor.bump();
        eat_digits(cursor, 10);
        true
    } else {
        false
    }
}

fn next_simple_token(cursor: &mut Cursor) -> LexedToken {
    if let Some(c) = cursor.first() {
        cursor.bump();
        match c {
            '\t' | '\u{000C}' | ' ' | '\\' => {
                if c == '\\' {
                    if cursor.eat('\r') {
                        cursor.eat('\n');
                    } else if cursor.eat('\n') {
                        // do nothing
                    } else {
                        return cursor.token(ERROR);
                    }
                }

                while let Some(c) = cursor.first() {
                    match c {
                        '\t' | '\u{000C}' | ' ' => {
                            cursor.bump();
                        }
                        '\\' => {
                            if cursor.second().is_some_and(|c| c == '\r') {
                                cursor.bump();
                                cursor.bump();
                                cursor.eat('\n');
                            } else if cursor.second().is_some_and(|c| c == '\n') {
                                cursor.bump();
                                cursor.bump();
                            } else {
                                break;
                            }
                        }
                        _ => {
                            break;
                        }
                    }
                }
                cursor.token(WHITESPACE)
            }
            '\n' => cursor.token(TRIVIAL_NEWLINE),
            '\r' => {
                cursor.eat('\n');
                cursor.token(TRIVIAL_NEWLINE)
            }
            '#' => {
                while cursor.first().is_some_and(|c| c != '\n' && c != '\r') {
                    cursor.bump();
                }
                cursor.token(COMMENT)
            }
            '|' => {
                while cursor.first().is_some_and(|c| c != '|') {
                    cursor.bump();
                }
                cursor.bump();
                cursor.token(IDENTIFIER)
            }
            c if is_id_start_char(c) => {
                while cursor.first().is_some_and(is_id_char) {
                    cursor.bump();
                }
                cursor.token(IDENTIFIER)
            }
            '"' | '\'' => {
                if !cursor.eat(c) {
                    // non-empty string
                    while let Some(e) = cursor.first() {
                        match e {
                            '\\' => {
                                cursor.bump();
                                cursor.bump();
                            }
                            '\n' | '\r' => break,
                            _ if e == c => {
                                cursor.bump();
                                break;
                            }
                            _ => {
                                cursor.bump();
                            }
                        }
                    }
                } else if !cursor.eat(c) {
                    // empty string
                } else {
                    // long string
                    while let Some(e) = cursor.first() {
                        match e {
                            '\\' => {
                                cursor.bump();
                                cursor.bump();
                            }
                            _ if e == c => {
                                cursor.bump();
                                if cursor.eat(c) && cursor.eat(c) {
                                    break;
                                }
                            }
                            _ => {
                                cursor.bump();
                            }
                        }
                    }
                }
                cursor.token(STRING_LITERAL)
            }
            '0'..='9' => {
                if c == '0' && cursor.first().is_some_and(|c| c == 'x') {
                    if cursor.second().is_some_and(|c| c.is_digit(16)) {
                        // hexadecimal
                        cursor.bump();
                        eat_digits(cursor, 16);
                        cursor.token(INTEGER_LITERAL)
                    } else {
                        // just '0'
                        cursor.token(INTEGER_LITERAL)
                    }
                } else {
                    // decimal
                    eat_digits(cursor, 10);
                    if let Some('.') = cursor.first() {
                        if cursor.second().is_some_and(|c| c.is_digit(10)) {
                            cursor.bump();
                            eat_digits(cursor, 10);
                            eat_exponent(cursor);
                            cursor.token(FLOAT_LITERAL)
                        } else {
                            cursor.token(INTEGER_LITERAL)
                        }
                    } else if eat_exponent(cursor) {
                        cursor.token(FLOAT_LITERAL)
                    } else {
                        cursor.token(INTEGER_LITERAL)
                    }
                }
            }
            '.' => {
                if cursor.eat('.') {
                    cursor.token(DOT_DOT)
                } else if cursor.first().is_some_and(|c| c.is_digit(10)) {
                    eat_digits(cursor, 10);
                    eat_exponent(cursor);
                    cursor.token(FLOAT_LITERAL)
                } else {
                    cursor.token(DOT)
                }
            }
            ',' => cursor.token(COMMA),
            '@' => cursor.token(AT),
            '(' => cursor.token(LEFT_PAREN),
            ')' => cursor.token(RIGHT_PAREN),
            '[' => cursor.token(LEFT_BRACKET),
            ']' => cursor.token(RIGHT_BRACKET),
            '?' => cursor.token(QUESTION),
            '+' => cursor.token(PLUS),
            '*' => cursor.token(STAR),
            '/' => cursor.token(SLASH),
            '%' => cursor.token(PERCENT),
            ':' => {
                if cursor.eat(':') {
                    cursor.token(COLON_COLON)
                } else {
                    cursor.token(COLON)
                }
            }
            '<' => {
                if cursor.eat('=') {
                    cursor.token(LESS_EQUAL)
                } else {
                    cursor.token(LESS)
                }
            }
            '>' => {
                if cursor.eat('=') {
                    cursor.token(GREATER_EQUAL)
                } else {
                    cursor.token(GREATER)
                }
            }
            '!' => {
                if cursor.eat('=') {
                    cursor.token(NOT_EQUAL)
                } else {
                    cursor.token(EXCLAMATION)
                }
            }
            '-' => {
                if cursor.eat('>') {
                    cursor.token(ARROW)
                } else {
                    cursor.token(MINUS)
                }
            }
            '=' => {
                if cursor.eat('>') {
                    cursor.token(FAT_ARROW)
                } else if cursor.eat('=') {
                    cursor.token(EQUAL)
                } else {
                    cursor.token(ASSIGN)
                }
            }
            _ => cursor.token(ERROR),
        }
    } else {
        cursor.token(EOF)
    }
}

pub struct Lexer<'a> {
    cursor: Cursor<'a>,
    tokens: VecDeque<LexedToken>,
}

impl Lexer<'_> {
    pub fn new(source: &str) -> Lexer {
        Lexer {
            cursor: Cursor::new(source),
            tokens: VecDeque::new(),
        }
    }

    fn bump(&mut self) -> LexedToken {
        match self.tokens.pop_front() {
            Some(token) => token,
            None => next_simple_token(&mut self.cursor),
        }
    }

    fn nth(&mut self, n: usize) -> &LexedToken {
        while self.tokens.len() <= n {
            let token = next_simple_token(&mut self.cursor);
            self.tokens.push_back(token);
        }
        &self.tokens[n]
    }

    pub fn next_token(&mut self) -> LexedToken {
        let token = self.bump();

        // glue sign token and adjacent numeric token to meet max munch rule
        match token.kind {
            PLUS => match self.nth(0).kind {
                FLOAT_LITERAL => {
                    let next_token = self.bump();
                    LexedToken {
                        kind: FLOAT_LITERAL,
                        length: token.length + next_token.length,
                    }
                }
                _ => token,
            },
            MINUS => match self.nth(0).kind {
                kind @ (INTEGER_LITERAL | FLOAT_LITERAL) => {
                    let next_token = self.bump();
                    LexedToken {
                        kind,
                        length: token.length + next_token.length,
                    }
                }
                _ => token,
            },
            _ => token,
        }
    }
}

#[cfg(test)]
mod tests {
    fn tokenize(source: &str) -> Vec<&str> {
        let mut result = Vec::new();
        let mut lexer = super::Lexer::new(source);
        let mut offset = 0;
        loop {
            let token = lexer.next_token();
            if token.kind == super::EOF {
                return result;
            }
            result.push(&source[offset..][..token.length]);
            offset += token.length;
        }
    }

    #[test]
    fn test_numeric() {
        assert_eq!(tokenize("0x"), ["0", "x"]);
        assert_eq!(tokenize("0x0"), ["0x0"]);

        assert_eq!(tokenize("123."), ["123", "."]);
        assert_eq!(tokenize("123.."), ["123", ".."]);
        assert_eq!(tokenize(".123"), [".123"]);
        assert_eq!(tokenize("..123"), ["..", "123"]);

        assert_eq!(tokenize("1.23"), ["1.23"]);
        assert_eq!(tokenize("+1.23"), ["+1.23"]);
        assert_eq!(tokenize("-1.23"), ["-1.23"]);

        assert_eq!(tokenize("1.23e456"), ["1.23e456"]);
        assert_eq!(tokenize("1.23e 456"), ["1.23", "e", " ", "456"]);
        assert_eq!(tokenize("1.23e+456"), ["1.23e+456"]);
        assert_eq!(tokenize("1.23ee+456"), ["1.23", "ee", "+", "456"]);
        assert_eq!(tokenize("1.23e +456"), ["1.23", "e", " ", "+", "456"]);
        assert_eq!(tokenize("1.23e-456"), ["1.23e-456"]);
        assert_eq!(tokenize("1.23ee-456"), ["1.23", "ee", "-456"]);
        assert_eq!(tokenize("1.23e -456"), ["1.23", "e", " ", "-456"]);
    }
}
