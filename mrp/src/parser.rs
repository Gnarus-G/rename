use crate::lexer::{Lexer, Token};

struct Parser<'a> {
    lexer: Lexer<'a>,
    token: Token,
    peek_token: Token,
}

#[derive(Debug, PartialEq)]
enum Expression {
    Literal(String),
}

#[derive(Debug, PartialEq)]
struct MatchExpression {
    expressions: Vec<Expression>,
}

impl<'a: 't, 't> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        let mut p = Self {
            lexer,
            token: Token::Eof,
            peek_token: Token::Eof,
        };
        p.advance();
        p.advance();
        p
    }

    fn advance<'s: 't>(&'s mut self) {
        self.token = self.peek_token.clone();
        self.peek_token = self.lexer.next();
    }

    fn parse_match_exp(&mut self) -> MatchExpression {
        let mut expressions = vec![];

        while self.token != Token::Eof {
            let exp = match self.token {
                Token::Literal(l) => self.parse_literal_exp(l),
                Token::Lparen => todo!(),
                Token::Rparen => todo!(),
                Token::DigitType => todo!(),
                Token::IntType => todo!(),
                Token::Ident(_) => todo!(),
                Token::Colon => todo!(),
                Token::Eof => todo!(),
            };

            expressions.push(exp);

            self.advance();
        }

        MatchExpression { expressions }
    }

    fn parse_literal_exp(&mut self, first_char: char) -> Expression {
        let mut lit = String::from(first_char);
        while let Token::Literal(ch) = self.peek_token {
            self.advance();
            lit.push(ch)
        }
        Expression::Literal(lit)
    }
}

mod test {
    use super::*;

    #[test]
    fn test_simple_match_expression() {
        let input = "abc";
        let mut p = Parser::new(Lexer::new(input));

        assert_eq!(
            p.parse_match_exp(),
            MatchExpression {
                expressions: vec![Expression::Literal("abc".to_string())]
            }
        );

        let input = "1234";
        let mut p = Parser::new(Lexer::new(input));

        assert_eq!(
            p.parse_match_exp(),
            MatchExpression {
                expressions: vec![Expression::Literal("1234".to_string())]
            }
        )
    }
}
