use itertools::Itertools as _;

use crate::{
    lexpr::Lexpr,
    tokenizer::{Span, Token, TokenKind, Tokenizer},
};

#[derive(Debug)]
enum ParseError {
    TokenizeError(crate::tokenizer::TokenizeError),
    UnexpectedToken {
        token: Token,
        expected: Option<TokenKind>,
    },
    UnexpectedEof {
        expected: Option<TokenKind>,
    },
}

type ParseResult<T> = Result<T, ParseError>;

struct Parser<'a> {
    tokenizer: Tokenizer<'a>,
}

#[derive(Debug, Clone)]
struct List(Vec<RightAssocExpr>);
#[derive(Debug, Clone)]
enum RightAssocExpr {
    RightAssocExpr {
        left: LeftAssocExpr,
        colon: Token,
        right: Box<RightAssocExpr>,
    },
    LeftAssocExpr(LeftAssocExpr),
}

#[derive(Debug, Clone)]
enum Sexp {
    List(Vec<Sexp>),
    Number(LiteralNumber),
    String(LiteralString),
    Name(LiteralName),
}

impl Sexp {
    fn stringify(&self) -> String {
        match self {
            Sexp::List(exprs) => {
                format!("({})", exprs.iter().map(|expr| expr.stringify()).join(" "))
            }
            Sexp::Number(number) => format!("{}", number.value),
            Sexp::String(string) => format!("{:#?}", string.value),
            Sexp::Name(name) => format!("{}", name.value),
        }
    }
}

#[derive(Debug, Clone)]
struct LiteralString {
    value: String,
    span: Span,
}

#[derive(Debug, Clone)]
struct LiteralName {
    value: String,
    span: Span,
}

#[derive(Debug, Clone)]
struct LiteralNumber {
    value: f64,
    span: Span,
}

impl RightAssocExpr {
    fn to_sexp(&self) -> Sexp {
        match self {
            RightAssocExpr::RightAssocExpr { left, colon, right } => {
                let left = left.to_sexp();
                let right = right.to_sexp();
                match left {
                    Sexp::List(exprs) => match exprs.split_first() {
                        Some((head, tail)) => Sexp::List(
                            Some(head.clone())
                                .into_iter()
                                .chain(tail.to_vec())
                                .chain(Some(right))
                                .collect(),
                        ),
                        None => Sexp::List(exprs),
                    },
                    _ => Sexp::List([left, right].to_vec()),
                }
            }
            RightAssocExpr::LeftAssocExpr(expr) => expr.to_sexp(),
        }
    }
}

#[derive(Debug, Clone)]
enum LeftAssocExpr {
    LeftAssocExpr {
        left: Box<LeftAssocExpr>,
        dot: Token,
        right: OperatorFunctionCallLike,
    },
    OperatorFunctionCallLike(OperatorFunctionCallLike),
}
impl LeftAssocExpr {
    fn to_sexp(&self) -> Sexp {
        match self {
            LeftAssocExpr::LeftAssocExpr { left, dot, right } => {
                let right = right.to_sexp();
                let left = left.to_sexp();
                match right {
                    Sexp::List(exprs) => match exprs.split_first() {
                        Some((head, tail)) => Sexp::List(
                            Some(head.clone())
                                .into_iter()
                                .chain(Some(left))
                                .chain(tail.to_vec())
                                .collect(),
                        ),
                        None => left,
                    },
                    _ => Sexp::List([right, left].to_vec()),
                }
            }
            LeftAssocExpr::OperatorFunctionCallLike(expr) => expr.to_sexp(),
        }
    }
}

#[derive(Debug, Clone)]
struct OperatorFunctionCallLike {
    head: OperatorFunctionCallLikeComponent,
    tail: Vec<OperatorFunctionCallLikeComponent>,
}
#[derive(Debug, Clone)]
enum OperatorFunctionCallLikeComponent {
    Operator(Operator),
    AlphanumericFunctionCallLike(AlphanumericFunctionCallLike),
}
impl OperatorFunctionCallLikeComponent {
    fn span(&self) -> Span {
        match self {
            OperatorFunctionCallLikeComponent::Operator(operator) => operator.span(),
            OperatorFunctionCallLikeComponent::AlphanumericFunctionCallLike(function_call_like) => {
                function_call_like.span()
            }
        }
    }
}

#[derive(Debug, Clone)]
struct Operator {
    representation: String,
    span: Span,
}
impl Operator {
    fn span(&self) -> Span {
        self.span
    }
}
impl OperatorFunctionCallLike {
    fn to_sexp(&self) -> Sexp {
        if self.tail.is_empty() {
            match &self.head {
                OperatorFunctionCallLikeComponent::Operator(operator) => Sexp::Name(LiteralName {
                    value: operator.representation.clone(),
                    span: operator.span,
                }),
                OperatorFunctionCallLikeComponent::AlphanumericFunctionCallLike(
                    function_call_like,
                ) => function_call_like.to_sexp(),
            }
        } else {
            let iter = Some(&self.head).into_iter().chain(self.tail.iter());
            let name = iter
                .clone()
                .map(|component| match component {
                    OperatorFunctionCallLikeComponent::Operator(operator) => {
                        operator.representation.clone()
                    }
                    _ => "_".to_string(),
                })
                .join("");
            let arguments = iter
                .filter_map(|expr| match expr {
                    OperatorFunctionCallLikeComponent::AlphanumericFunctionCallLike(
                        function_call_like,
                    ) => Some(function_call_like.to_sexp()),
                    _ => None,
                })
                .collect_vec();
            Sexp::List(
                [Sexp::Name(LiteralName {
                    value: name,
                    span: self
                        .tail
                        .last()
                        .map(|last| self.head.span().join(&last.span()))
                        .unwrap_or(self.head.span()),
                })]
                .into_iter()
                .chain(arguments)
                .collect(),
            )
        }
    }
}
#[derive(Debug, Clone)]
enum AlphanumericFunctionCallLike {
    FunctionCallLike(FunctionCallLike),
    AtomicExpr(AtomicExpr),
}
impl AlphanumericFunctionCallLike {
    fn to_sexp(&self) -> Sexp {
        match self {
            AlphanumericFunctionCallLike::FunctionCallLike(function_call_like) => {
                function_call_like.to_sexp()
            }
            AlphanumericFunctionCallLike::AtomicExpr(expr) => expr.to_sexp(),
        }
    }

    fn span(&self) -> Span {
        match self {
            AlphanumericFunctionCallLike::FunctionCallLike(function_call_like) => {
                function_call_like.span()
            }
            AlphanumericFunctionCallLike::AtomicExpr(expr) => expr.span(),
        }
    }
}

#[derive(Debug, Clone)]
struct FunctionCallLike {
    head: AtomicExpr,
    tail: Vec<AtomicExpr>,
}
impl FunctionCallLike {
    fn to_sexp(&self) -> Sexp {
        let iter = Some(&self.head).into_iter().chain(self.tail.iter());
        let name = iter
            .clone()
            .map(|expr| match expr {
                AtomicExpr::Name(name) => name.value.clone(),
                _ => "_".to_string(),
            })
            .join("");
        let arguments = iter
            .filter_map(|expr| match expr {
                AtomicExpr::Name(_) => None,
                _ => Some(expr.to_sexp()),
            })
            .collect_vec();
        Sexp::List(
            [Sexp::Name(LiteralName {
                value: name,
                span: self
                    .tail
                    .last()
                    .map(|last| self.head.span().join(&last.span()))
                    .unwrap_or(self.head.span()),
            })]
            .into_iter()
            .chain(arguments)
            .collect(),
        )
    }

    fn span(&self) -> Span {
        match self.tail.last() {
            Some(tail) => self.head.span().join(&tail.span()),
            None => self.head.span(),
        }
    }
}

#[derive(Debug, Clone)]
enum AtomicExpr {
    String(LiteralString),
    Number(LiteralNumber),
    Parenthesized(ParenthesizedExpr),
    Name(LiteralName),
}
impl AtomicExpr {
    fn to_sexp(&self) -> Sexp {
        match self {
            AtomicExpr::String(string) => Sexp::String(string.clone()),
            AtomicExpr::Number(number) => Sexp::Number(number.clone()),
            AtomicExpr::Parenthesized(expr) => expr.list.to_sexp(),
            AtomicExpr::Name(name) => Sexp::Name(name.clone()),
        }
    }

    fn span(&self) -> Span {
        match self {
            AtomicExpr::String(string) => string.span,
            AtomicExpr::Number(number) => number.span,
            AtomicExpr::Parenthesized(parenthesized) => {
                parenthesized.open.span.join(&parenthesized.close.span)
            }
            AtomicExpr::Name(name) => name.span,
        }
    }
}

#[derive(Debug, Clone)]
struct ParenthesizedExpr {
    open: Token,
    list: List,
    close: Token,
}

impl List {
    fn to_sexp(&self) -> Sexp {
        let inner = self.0.iter().map(|expr| expr.to_sexp()).collect_vec();
        Sexp::List(inner)
    }
}

impl<'a> Parser<'a> {
    fn new(input_text: &'a str) -> Parser<'a> {
        Self {
            tokenizer: Tokenizer::new(input_text),
        }
    }
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
        let expr = self.parse_operator_function_call_like()?;
        self.try_parse_left_assoc_expr(LeftAssocExpr::OperatorFunctionCallLike(expr))
    }

    fn try_parse_left_assoc_expr(
        &mut self,
        leading: LeftAssocExpr,
    ) -> Result<LeftAssocExpr, ParseError> {
        if let Some(token) = self.try_eat_token(TokenKind::Dot)? {
            let right = self.parse_operator_function_call_like()?;
            self.try_parse_left_assoc_expr(LeftAssocExpr::LeftAssocExpr {
                left: Box::new(leading),
                dot: token,
                right,
            })
        } else {
            Ok(leading)
        }
    }

    fn parse_operator_function_call_like(&mut self) -> ParseResult<OperatorFunctionCallLike> {
        let head = self.parse_operator_function_call_like_component()?;
        let tail = {
            let mut tail = vec![];
            loop {
                match self.peek_token()? {
                    Some(Token {
                        kind:
                            TokenKind::Comma
                            | TokenKind::Dot
                            | TokenKind::Colon
                            | TokenKind::RightBrace
                            | TokenKind::RightBracket
                            | TokenKind::RightParenthesis,
                        ..
                    })
                    | None => break tail,
                    _ => {
                        tail.push(self.parse_operator_function_call_like_component()?);
                    }
                }
            }
        };
        Ok(OperatorFunctionCallLike { head, tail })
    }

    fn parse_alphanumeric_function_call_like(
        &mut self,
    ) -> ParseResult<AlphanumericFunctionCallLike> {
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
            Ok(AlphanumericFunctionCallLike::AtomicExpr(head))
        } else {
            Ok(AlphanumericFunctionCallLike::FunctionCallLike(
                FunctionCallLike { head, tail },
            ))
        }
    }

    fn parse_atomic_expr(&mut self) -> ParseResult<AtomicExpr> {
        if let Some(token) = self.next_token()? {
            let expr = match token.kind {
                TokenKind::Identifier(value) => AtomicExpr::Name(LiteralName {
                    value,
                    span: token.span,
                }),
                TokenKind::StringLiteral(value) => AtomicExpr::String(LiteralString {
                    value,
                    span: token.span,
                }),
                TokenKind::NumberLiteral(value) => AtomicExpr::Number(LiteralNumber {
                    value,
                    span: token.span,
                }),
                TokenKind::LeftParenthesis => AtomicExpr::Parenthesized(
                    self.parse_list_ending_with(token, TokenKind::RightParenthesis)?,
                ),
                TokenKind::LeftBrace => AtomicExpr::Parenthesized(
                    self.parse_list_ending_with(token, TokenKind::RightBrace)?,
                ),
                TokenKind::LeftBracket => AtomicExpr::Parenthesized(
                    self.parse_list_ending_with(token, TokenKind::RightBracket)?,
                ),
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        token,
                        expected: None,
                    })
                }
            };
            Ok(expr)
        } else {
            Err(ParseError::UnexpectedEof { expected: None })
        }
    }

    fn peek_token(&mut self) -> ParseResult<Option<Token>> {
        self.tokenizer
            .peek_token()
            .map_err(ParseError::TokenizeError)
    }

    fn eat_token(&mut self, expected_token_kind: TokenKind) -> ParseResult<Token> {
        match self.next_token()? {
            Some(token) => {
                if token.kind != expected_token_kind {
                    Err(ParseError::UnexpectedToken {
                        token,
                        expected: Some(expected_token_kind),
                    })
                } else {
                    Ok(token)
                }
            }
            None => Err(ParseError::UnexpectedEof {
                expected: Some(expected_token_kind),
            }),
        }
    }

    fn try_eat_token(&mut self, token_kind: TokenKind) -> ParseResult<Option<Token>> {
        match self.peek_token()? {
            Some(token) if token.kind == token_kind => {
                self.next_token()?;
                Ok(Some(token))
            }
            _ => Ok(None),
        }
    }

    fn parse_list_ending_with(
        &mut self,
        open: Token,
        close_kind: TokenKind,
    ) -> ParseResult<ParenthesizedExpr> {
        let list = self.parse_list()?;
        let close = self.eat_token(close_kind)?;
        Ok(ParenthesizedExpr { open, close, list })
    }

    fn parse_operator_function_call_like_component(
        &mut self,
    ) -> ParseResult<OperatorFunctionCallLikeComponent> {
        match self.peek_token()? {
            Some(Token {
                kind: TokenKind::Operator(operator),
                span,
                ..
            }) => {
                self.next_token()?;
                Ok(OperatorFunctionCallLikeComponent::Operator(Operator {
                    span,
                    representation: operator,
                }))
            }
            _ => Ok(
                OperatorFunctionCallLikeComponent::AlphanumericFunctionCallLike(
                    self.parse_alphanumeric_function_call_like()?,
                ),
            ),
        }
    }
}

#[cfg(test)]
mod test_parse {
    use super::{ParseResult, Parser};

    #[test]
    fn operator_1() -> ParseResult<()> {
        let input = "n *: n - 1 .factorial";
        let mut parser = Parser::new(input);
        let list = parser.parse_list()?;
        println!("{}", input);
        println!("{}", list.to_sexp().stringify());
        Ok(())
    }

    #[test]
    fn case_1() -> ParseResult<()> {
        let input = "x <= y < z";
        let mut parser = Parser::new(input);
        let list = parser.parse_list()?;
        println!("{}", input);
        println!("{}", list.to_sexp().stringify());
        Ok(())
    }

    #[test]
    fn case_2() -> ParseResult<()> {
        let input = "def (n .factorial): if (n < 2) then 1 else: n *: n - 1 .factorial";
        let mut parser = Parser::new(input);
        let list = parser.parse_list()?;
        println!("{}", input);
        println!("{}", list.to_sexp().stringify());
        Ok(())
    }
}

/*
def ((n: int) .factorial):
    if (n < 2) then
        1
    else:
        n *: n - 1 .!

(def (factorial (n int)) (if-then-else (< n 2) 1 (* n (! (- n 1)))))
*/
