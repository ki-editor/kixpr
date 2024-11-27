use crate::{
    lexpr::Lexpr,
    tokenizer::{Token, TokenKind, Tokenizer},
};

enum ParseError {
    TokenizeError(crate::tokenizer::TokenizeError),
}

type ParseResult<T> = Result<T, ParseError>;

struct Parser<'a> {
    tokenizer: Tokenizer<'a>,
}

struct List(Vec<RightAssocExpr>);
enum RightAssocExpr {
    RightAssocExpr {
        left: LeftAssocExpr,
        colon: Token,
        right: Box<RightAssocExpr>,
    },
    LeftAssocExpr(LeftAssocExpr),
}

enum LeftAssocExpr {
    LeftAssocExpr {
        left: Box<LeftAssocExpr>,
        dot: Token,
        right: NoAssocExpr,
    },
    NoAssocExpr(NoAssocExpr),
}

enum NoAssocExpr {
    FunctionCallLike(FunctionCallLike),
    AtomicExpr(AtomicExpr),
}

struct FunctionCallLike {
    head: AtomicExpr,
    tail: Vec<AtomicExpr>,
}
enum AtomicExpr {
    String(Token),
    Number(Token),
    Parenthesized {
        open: Token,
        expr: List,
        close: Token,
    },
    Name(Vec<Token>),
}

impl<'a> Parser<'a> {
    fn next_token(&mut self) -> ParseResult<Option<Token>> {
        self.tokenizer
            .next_token()
            .map_err(|err| ParseError::TokenizeError(err))
    }
    fn parse_list(&mut self) -> ParseResult<List> {
        let mut exprs = vec![];
        loop {
            let right_assoc_expr = self.parse_right_assoc_expr()?;
            exprs.push(right_assoc_expr);
            if self.try_eat_token(TokenKind::Comma)?.is_none() {
                return Ok(List(exprs));
            }
        }
    }

    fn parse_right_assoc_expr(&mut self) -> ParseResult<RightAssocExpr> {
        let left = self.parse_left_assoc_expr()?;
        if let Some(token) = self.try_eat_token(TokenKind::Colon)? {
            let right = self.parse_right_assoc_expr()?;
            Ok(RightAssocExpr::RightAssocExpr {
                left,
                colon: token,
                right: Box::new(right),
            })
        } else {
            Ok(RightAssocExpr::LeftAssocExpr(left))
        }
    }

    fn parse_left_assoc_expr(&mut self) -> ParseResult<LeftAssocExpr> {
        let expr = self.parse_no_assoc_expr()?;
        self.try_parse_left_assoc_expr(LeftAssocExpr::NoAssocExpr(expr))
    }

    fn try_parse_left_assoc_expr(
        &mut self,
        leading: LeftAssocExpr,
    ) -> Result<LeftAssocExpr, ParseError> {
        if let Some(token) = self.try_eat_token(TokenKind::Dot)? {
            let right = self.parse_no_assoc_expr()?;
            self.try_parse_left_assoc_expr(LeftAssocExpr::LeftAssocExpr {
                left: Box::new(leading),
                dot: token,
                right,
            })
        } else {
            Ok(leading)
        }
    }

    fn parse_no_assoc_expr(&mut self) -> ParseResult<NoAssocExpr> {
        let head = self.parse_atomic_expr()?;
        let tail = {
            let mut tail = vec![];
            loop {
                match self.peek_token()? {
                    Some(Token {
                        kind:
                            TokenKind::Identifier(_)
                            | TokenKind::StringLiteral(_)
                            | TokenKind::NumberLiteral(_)
                            | TokenKind::LeftBrace
                            | TokenKind::LeftParenthesis
                            | TokenKind::LeftBracket,
                        ..
                    }) => {
                        tail.push(self.parse_atomic_expr()?);
                    }
                    _ => break tail,
                }
            }
        };
        if tail.is_empty() {
            Ok(NoAssocExpr::AtomicExpr(head))
        } else {
            Ok(NoAssocExpr::FunctionCallLike(FunctionCallLike {
                head,
                tail,
            }))
        }
    }

    fn parse_atomic_expr(&self) -> ParseResult<AtomicExpr> {
        while let Some(token) = self.next_token()? {
            match token.kind {
                TokenKind::Identifier() => AtomicExpr(),
                TokenKind::StringLiteral() => todo!(),
                TokenKind::NumberLiteral() => todo!(),
                TokenKind::LeftParenthesis => todo!(),
                TokenKind::RightParenthesis => todo!(),
                TokenKind::LeftBrace => todo!(),
                TokenKind::RightBrace => todo!(),
                TokenKind::LeftBracket => todo!(),
                TokenKind::RightBracket => todo!(),
                TokenKind::Colon => todo!(),
                TokenKind::Dot => todo!(),
                TokenKind::Comma => todo!(),
            }
        }
    }

    fn peek_token(&mut self) -> ParseResult<Option<Token>> {
        todo!()
    }

    fn eat_token(&mut self, colon: TokenKind) -> _ {
        todo!()
    }

    fn try_eat_token(&self, comma: TokenKind) -> ParseResult<Option<Token>> {
        todo!()
    }
}

fn parse(input_text: &str) -> ParseResult {
    let mut tokenizer = Tokenizer::new(input_text);
}

/*
def ((n: int) .factorial):
    if (n < 2) then
        1
    else:
        n *: n - 1 .!

(def (factorial (n int)) (if-then-else (< n 2) 1 (* n (! (- n 1)))))
*/
