use std::fmt::Display;

#[derive(Debug, PartialEq, Clone)]
pub enum Token<'s> {
    Literal(char),
    Lparen,
    Rparen,
    DigitType,
    IntType,
    Ident(&'s str),
    Colon,
    Arrow,
    End,
}

impl<'s> Display for Token<'s> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            Token::Literal(l) => write!(f, "{l}"),
            Token::Lparen => write!(f, "("),
            Token::Rparen => write!(f, ")"),
            Token::DigitType => write!(f, "dig"),
            Token::IntType => write!(f, "int"),
            Token::Ident(i) => write!(f, "{i}"),
            Token::Colon => write!(f, ":"),
            Token::Arrow => write!(f, "->"),
            Token::End => write!(f, "\0"),
        };
    }
}

#[derive(Debug, PartialEq)]
enum LexingMode {
    String,
    Char,
}

const NO_CHAR: char = 0 as char;

#[derive(Debug)]
pub struct Lexer<'a> {
    input: &'a str,
    ch: char,
    position: usize,
    read_position: usize,
    mode: LexingMode,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut l = Self {
            input,
            ch: NO_CHAR,
            position: 0,
            read_position: 0,
            mode: LexingMode::Char,
        };
        l.read_char();
        l
    }

    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = NO_CHAR;
        } else {
            self.ch = self.input.as_bytes()[self.read_position].into();
        }
        self.position = self.read_position;
        self.read_position += 1;
    }

    fn peek_char(&self) -> char {
        if self.read_position >= self.input.len() {
            return NO_CHAR;
        } else {
            return self.input.as_bytes()[self.read_position].into();
        }
    }

    fn read_while<P: Fn(&char) -> bool>(&mut self, predicate: P) -> &'a str {
        let start_pos = self.position;

        while predicate(&self.ch) {
            self.read_char();
        }

        return &self.input[start_pos..self.position];
    }

    pub fn next(&mut self) -> Token<'a> {
        let t = match self.ch {
            '(' => Token::Lparen,
            ':' => Token::Colon,
            ')' => Token::Rparen,
            '-' if self.peek_char() == '>' => {
                self.read_char();
                Token::Arrow
            }
            c if c == NO_CHAR => Token::End,
            c if c.is_ascii_alphabetic() && self.mode == LexingMode::String => {
                let str = self.read_while(|c| c.is_ascii_alphabetic());
                return match str {
                    "dig" => Token::DigitType,
                    "int" => Token::IntType,
                    str => Token::Ident(str),
                };
            }
            c => Token::Literal(c),
        };

        self.update_mode();

        self.read_char();

        t
    }

    fn update_mode(&mut self) {
        if ['(', ':'].contains(&self.ch) {
            self.mode = LexingMode::String;
        } else if self.ch == ')' {
            self.mode = LexingMode::Char;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_char_sequence() {
        let mut l = Lexer::new("abcna");

        assert_eq!(l.next(), Token::Literal('a'));
        assert_eq!(l.next(), Token::Literal('b'));
        assert_eq!(l.next(), Token::Literal('c'));
        assert_eq!(l.next(), Token::Literal('n'));
        assert_eq!(l.next(), Token::Literal('a'));
    }

    #[test]
    fn one_literal_and_one_digit_capture() {
        let mut l = Lexer::new("a5(d:dig)");
        assert_eq!(l.next(), Token::Literal('a'));
        assert_eq!(l.next(), Token::Literal('5'));
        assert_eq!(l.next(), Token::Lparen);
        assert_eq!(l.next(), Token::Ident("d"));
        assert_eq!(l.next(), Token::Colon);
        assert_eq!(l.next(), Token::DigitType);
        assert_eq!(l.next(), Token::Rparen);
    }

    #[test]
    fn one_literal_letter_and_one_integer_capture() {
        let mut l = Lexer::new("z(i:int)");
        assert_eq!(l.next(), Token::Literal('z'));
        assert_eq!(l.next(), Token::Lparen);
        assert_eq!(l.next(), Token::Ident("i"));
        assert_eq!(l.next(), Token::Colon);
        assert_eq!(l.next(), Token::IntType);
        assert_eq!(l.next(), Token::Rparen);
    }

    #[test]
    fn two_intersperced_captures() {
        let mut l = Lexer::new("iawe10(x:dig)zap(i:int)");

        assert_eq!(l.next(), Token::Literal('i'));
        assert_eq!(l.next(), Token::Literal('a'));
        assert_eq!(l.next(), Token::Literal('w'));
        assert_eq!(l.next(), Token::Literal('e'));
        assert_eq!(l.next(), Token::Literal('1'));
        assert_eq!(l.next(), Token::Literal('0'));
        assert_eq!(l.next(), Token::Lparen);
        assert_eq!(l.next(), Token::Ident("x"));
        assert_eq!(l.next(), Token::Colon);
        assert_eq!(l.next(), Token::DigitType);
        assert_eq!(l.next(), Token::Rparen);

        assert_eq!(l.next(), Token::Literal('z'));
        assert_eq!(l.next(), Token::Literal('a'));
        assert_eq!(l.next(), Token::Literal('p'));
        assert_eq!(l.next(), Token::Lparen);
        assert_eq!(l.next(), Token::Ident("i"));
        assert_eq!(l.next(), Token::Colon);
        assert_eq!(l.next(), Token::IntType);
        assert_eq!(l.next(), Token::Rparen);
    }

    #[test]
    fn two_consecutive_captures() {
        let mut l = Lexer::new("a5(d:dig)(num:int)");
        assert_eq!(l.next(), Token::Literal('a'));
        assert_eq!(l.next(), Token::Literal('5'));
        assert_eq!(l.next(), Token::Lparen);
        assert_eq!(l.next(), Token::Ident("d"));
        assert_eq!(l.next(), Token::Colon);
        assert_eq!(l.next(), Token::DigitType);
        assert_eq!(l.next(), Token::Rparen);

        assert_eq!(l.next(), Token::Lparen);
        assert_eq!(l.next(), Token::Ident("num"));
        assert_eq!(l.next(), Token::Colon);
        assert_eq!(l.next(), Token::IntType);
        assert_eq!(l.next(), Token::Rparen);
    }

    #[test]
    fn simple_match_and_replacement() {
        let mut l = Lexer::new("a(n:dig)->(n)b");
        assert_eq!(l.next(), Token::Literal('a'));
        assert_eq!(l.next(), Token::Lparen);
        assert_eq!(l.next(), Token::Ident("n"));
        assert_eq!(l.next(), Token::Colon);
        assert_eq!(l.next(), Token::DigitType);
        assert_eq!(l.next(), Token::Rparen);
        assert_eq!(l.next(), Token::Arrow);
        assert_eq!(l.next(), Token::Lparen);
        assert_eq!(l.next(), Token::Ident("n"));
        assert_eq!(l.next(), Token::Rparen);
        assert_eq!(l.next(), Token::Literal('b'));
    }
}
