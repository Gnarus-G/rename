use std::{
    fmt::Display,
    ops::{Deref, Range},
};

#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    Literal,
    Lparen,
    Rparen,
    DigitType,
    IntType,
    Ident,
    Colon,
    Arrow,
    End,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenText<'t> {
    Slice(&'t str),
    Empty,
}

impl<'t> Deref for TokenText<'t> {
    type Target = &'t str;

    fn deref(&self) -> &Self::Target {
        match self {
            TokenText::Slice(s) => &s,
            TokenText::Empty => &"",
        }
    }
}

impl<'a> TokenText<'a> {
    pub fn len(&self) -> usize {
        match self {
            TokenText::Slice(s) => s.len(),
            TokenText::Empty => 0,
        }
    }
}

impl<'t> Display for TokenText<'t> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TokenText::Slice(s) => s,
                TokenText::Empty => "",
            }
        )
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token<'t> {
    pub kind: TokenKind,
    pub text: TokenText<'t>,
    pub start: usize,
}

#[derive(Debug)]
pub struct Lexer<'a> {
    input: &'a [u8],
    position: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input: input.as_bytes(),
            position: 0,
        }
    }

    fn input_slice(&self, range: Range<usize>) -> &'a str {
        std::str::from_utf8(&self.input[range]).expect("input to only contain utf-8 characters")
    }

    fn char_at(&self, position: usize) -> Option<&u8> {
        if position < self.input.len() {
            return Some(&self.input[position]);
        }
        return None;
    }

    fn ch(&self) -> Option<&u8> {
        self.char_at(self.position)
    }

    fn step(&mut self) {
        self.position += 1;
    }

    fn peek_char(&self) -> Option<&u8> {
        self.char_at(self.position + 1)
    }

    fn if_peek(&self, ch: u8) -> bool {
        match self.peek_char() {
            Some(c) => *c == ch,
            None => false,
        }
    }

    fn if_previous(&self, ch: u8) -> bool {
        if self.position == 0 {
            return false;
        }
        match self.char_at(self.position - 1) {
            Some(c) => *c == ch,
            None => false,
        }
    }

    /// Assumes that the character at the current position, immediately before calling
    /// this function is also true the predicate function given.
    fn read_while<P: Fn(&u8) -> bool>(&mut self, predicate: P) -> (usize, usize) {
        let start_pos = self.position;

        while match self.peek_char() {
            Some(c) => predicate(c),
            None => false,
        } {
            self.step();
        }

        return (start_pos, self.position + 1);
    }

    pub fn next_token(&mut self) -> Token<'a> {
        let t = match self.ch() {
            Some(ch) => match ch {
                b'(' => self.char_token(TokenKind::Lparen),
                b')' => self.char_token(TokenKind::Rparen),
                b'-' if self.if_peek(b'>') => {
                    let t = self.char_token(TokenKind::Arrow);
                    self.step();
                    t
                }
                b':' => self.char_token(TokenKind::Colon),
                _ if self.if_previous(b':') => self.type_token(),
                _ if self.if_previous(b'(') => self.identifier_token(),
                _ => self.literal(),
            },
            None => Token {
                kind: TokenKind::End,
                text: TokenText::Empty,
                start: self.position,
            },
        };

        self.step();

        t
    }

    fn type_token(&mut self) -> Token<'a> {
        let start = self.position;
        let (s, e) = self.read_while(|c| c.is_ascii_alphabetic());
        let slice = self.input_slice(s..e);
        let kind = match slice {
            "dig" => TokenKind::DigitType,
            "int" => TokenKind::IntType,
            _ => unreachable!(),
        };
        Token {
            kind,
            text: TokenText::Empty,
            start,
        }
    }

    fn identifier_token(&mut self) -> Token<'a> {
        let start = self.position;
        let (s, e) = self.read_while(|c| c.is_ascii_alphabetic());
        let slice = self.input_slice(s..e);

        Token {
            kind: TokenKind::Ident,
            text: TokenText::Slice(slice),
            start,
        }
    }

    fn literal(&mut self) -> Token<'a> {
        let start = self.position;
        let (s, e) = self.read_while(|c| match c {
            b'(' | b')' | b':' | b'-' => false,
            _ => true,
        });
        Token {
            kind: TokenKind::Literal,
            text: TokenText::Slice(self.input_slice(s..e)),
            start,
        }
    }

    fn char_token(&self, kind: TokenKind) -> Token<'a> {
        Token {
            kind,
            text: TokenText::Empty,
            start: self.position,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TokenKind::*;
    use super::*;

    fn token<'s>(kind: TokenKind, start: usize) -> Token<'s> {
        Token {
            kind,
            text: TokenText::Empty,
            start,
        }
    }

    fn token_string(kind: TokenKind, text: &str, start: usize) -> Token {
        Token {
            kind,
            text: TokenText::Slice(text),
            start,
        }
    }

    #[test]
    fn one_char_sequence() {
        let mut l = Lexer::new("abcna");
        assert_eq!(l.next_token(), token_string(Literal, "abcna", 0));
    }

    #[test]
    fn one_literal_and_one_digit_capture() {
        let mut l = Lexer::new("a5(d:dig)");

        assert_eq!(l.next_token(), token_string(Literal, "a5", 0));
        assert_eq!(l.next_token(), token(Lparen, 2));
        assert_eq!(l.next_token(), token_string(Ident, "d", 3));
        assert_eq!(l.next_token(), token(Colon, 4));
        assert_eq!(l.next_token(), token(DigitType, 5));
        assert_eq!(l.next_token(), token(Rparen, 8));
    }

    #[test]
    fn one_literal_letter_and_one_integer_capture() {
        let mut l = Lexer::new("z(i:int)");
        assert_eq!(l.next_token(), token_string(Literal, "z", 0));
        assert_eq!(l.next_token(), token(Lparen, 1));
        assert_eq!(l.next_token(), token_string(Ident, "i", 2));
        assert_eq!(l.next_token(), token(Colon, 3));
        assert_eq!(l.next_token(), token(IntType, 4));
        assert_eq!(l.next_token(), token(Rparen, 7));
    }

    #[test]
    fn two_intersperced_captures() {
        let mut l = Lexer::new("iawe10(x:dig)zap(i:int)");

        assert_eq!(l.next_token(), token_string(Literal, "iawe10", 0));
        assert_eq!(l.next_token(), token(Lparen, 6));
        assert_eq!(l.next_token(), token_string(Ident, "x", 7));
        assert_eq!(l.next_token(), token(Colon, 8));
        assert_eq!(l.next_token(), token(DigitType, 9));
        assert_eq!(l.next_token(), token(Rparen, 12));

        assert_eq!(l.next_token(), token_string(Literal, "zap", 13));
        assert_eq!(l.next_token(), token(Lparen, 16));
        assert_eq!(l.next_token(), token_string(Ident, "i", 17));
        assert_eq!(l.next_token(), token(Colon, 18));
        assert_eq!(l.next_token(), token(IntType, 19));
        assert_eq!(l.next_token(), token(Rparen, 22));
    }

    #[test]
    fn two_consecutive_captures() {
        let mut l = Lexer::new("a5(d:dig)(num:int)");
        assert_eq!(l.next_token(), token_string(Literal, "a5", 0));
        assert_eq!(l.next_token(), token(Lparen, 2));
        assert_eq!(l.next_token(), token_string(Ident, "d", 3));
        assert_eq!(l.next_token(), token(Colon, 4));
        assert_eq!(l.next_token(), token(DigitType, 5));
        assert_eq!(l.next_token(), token(Rparen, 8));

        assert_eq!(l.next_token(), token(Lparen, 9));
        assert_eq!(l.next_token(), token_string(Ident, "num", 10));
        assert_eq!(l.next_token(), token(Colon, 13));
        assert_eq!(l.next_token(), token(IntType, 14));
        assert_eq!(l.next_token(), token(Rparen, 17));
    }

    #[test]
    fn simple_match_and_replacement() {
        let mut l = Lexer::new("a(n:dig)->(n)b");
        assert_eq!(l.next_token(), token_string(Literal, "a", 0));
        assert_eq!(l.next_token(), token(Lparen, 1));
        assert_eq!(l.next_token(), token_string(Ident, "n", 2));
        assert_eq!(l.next_token(), token(Colon, 3));
        assert_eq!(l.next_token(), token(DigitType, 4));
        assert_eq!(l.next_token(), token(Rparen, 7));
        assert_eq!(l.next_token(), token(Arrow, 8));
        assert_eq!(l.next_token(), token(Lparen, 10));
        assert_eq!(l.next_token(), token_string(Ident, "n", 11));
        assert_eq!(l.next_token(), token(Rparen, 12));
        assert_eq!(l.next_token(), token_string(Literal, "b", 13));
    }
}
