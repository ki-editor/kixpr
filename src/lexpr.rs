use crate::tokenizer::Token;

pub(crate) enum Lexpr {
    LeftAssociativeChaining {
        left: Box<Lexpr>,
        dot: Token,
        right: Box<Lexpr>,
    },
    RightAssociativeChaining {
        left: Box<Lexpr>,
        colon: Token,
        right: Box<Lexpr>,
    },
    String(Token),
    Number(Token),
    Variable(Vec<Token>),
    List(Vec<Lexpr>),
    Call(Vec<Lexpr>),
}
