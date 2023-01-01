#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Literal(char),
    Lparen,
    Rparen,
    DigitType,
    IntType,
    Ident(String),
    Colon,
    Eof,
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

    fn read_while<P: Fn(&char) -> bool>(&mut self, predicate: P) -> &str {
        let start_pos = self.position;

        while predicate(&self.ch) {
            self.read_char();
        }

        return &self.input[start_pos..self.position];
    }

    pub fn next(&mut self) -> Token {
        let t = match self.ch {
            '(' => Token::Lparen,
            ':' => Token::Colon,
            ')' => Token::Rparen,
            c if c == NO_CHAR => Token::Eof,
            c if c.is_ascii_alphabetic() && self.mode == LexingMode::String => {
                let str = self.read_while(|c| c.is_ascii_alphabetic());
                return match str {
                    "dig" => Token::DigitType,
                    "int" => Token::IntType,
                    str => Token::Ident(str.to_string()),
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

fn to_digit(c: &char) -> Option<u8> {
    match c {
        '0' => Some(0),
        '1' => Some(1),
        '2' => Some(2),
        '3' => Some(3),
        '4' => Some(4),
        '5' => Some(5),
        '6' => Some(6),
        '7' => Some(7),
        '8' => Some(8),
        '9' => Some(9),
        _ => None,
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
        assert_eq!(l.next(), Token::Ident("d".to_string()));
        assert_eq!(l.next(), Token::Colon);
        assert_eq!(l.next(), Token::DigitType);
        assert_eq!(l.next(), Token::Rparen);
    }

    #[test]
    fn one_literal_letter_and_one_integer_capture() {
        let mut l = Lexer::new("z(i:int)");
        assert_eq!(l.next(), Token::Literal('z'));
        assert_eq!(l.next(), Token::Lparen);
        assert_eq!(l.next(), Token::Ident("i".to_string()));
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
        assert_eq!(l.next(), Token::Ident("x".to_string()));
        assert_eq!(l.next(), Token::Colon);
        assert_eq!(l.next(), Token::DigitType);
        assert_eq!(l.next(), Token::Rparen);

        assert_eq!(l.next(), Token::Literal('z'));
        assert_eq!(l.next(), Token::Literal('a'));
        assert_eq!(l.next(), Token::Literal('p'));
        assert_eq!(l.next(), Token::Lparen);
        assert_eq!(l.next(), Token::Ident("i".to_string()));
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
        assert_eq!(l.next(), Token::Ident("d".to_string()));
        assert_eq!(l.next(), Token::Colon);
        assert_eq!(l.next(), Token::DigitType);
        assert_eq!(l.next(), Token::Rparen);

        assert_eq!(l.next(), Token::Lparen);
        assert_eq!(l.next(), Token::Ident("num".to_string()));
        assert_eq!(l.next(), Token::Colon);
        assert_eq!(l.next(), Token::IntType);
        assert_eq!(l.next(), Token::Rparen);
    }
}
