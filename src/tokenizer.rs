use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Position {
    pub line_number: usize,
    pub column_number: usize,
    pub character_index: usize,
}
impl Position {
    fn min(&self, other: &Position) -> Position {
        Position {
            line_number: self.line_number.min(other.line_number),
            column_number: self.column_number.min(other.column_number),
            character_index: self.character_index.min(other.character_index),
        }
    }
    fn max(&self, other: &Position) -> Position {
        Position {
            line_number: self.line_number.max(other.line_number),
            column_number: self.column_number.max(other.column_number),
            character_index: self.character_index.max(other.character_index),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}
impl Span {
    pub(crate) fn join(&self, other: &Span) -> Span {
        Span {
            start: self.start.min(&other.start),
            end: self.end.max(&other.end),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    Operator(String),
    Identifier(String),
    StringLiteral(String),
    NumberLiteral(f64),
    LeftParenthesis,
    RightParenthesis,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Colon,
    Dot,
    Comma,
}

pub struct Tokenizer<'a> {
    input_characters: Peekable<Chars<'a>>,
    current_position: Position,
    cached_next_token: Option<Token>,
}

#[derive(Debug)]
pub(crate) enum TokenizeError {
    UnexpectedCharacter(char),
    InvalidEscapeSequence(char),
    UnterminatedStringLiteral,
    InvalidNumberFormatMultipleDecimalPoints,
    FailedToParseNumber(String),
}

impl<'a> Tokenizer<'a> {
    pub fn new(input_text: &'a str) -> Self {
        Self {
            input_characters: input_text.chars().peekable(),
            current_position: Position {
                line_number: 1,
                column_number: 1,
                character_index: 0,
            },
            cached_next_token: None,
        }
    }

    fn advance_position(&mut self, character: char) {
        if character == '\n' {
            self.current_position.line_number += 1;
            self.current_position.column_number = 1;
        } else {
            self.current_position.column_number += 1;
        }
        self.current_position.character_index += 1;
    }

    fn consume_while<Predicate>(&mut self, predicate: Predicate) -> String
    where
        Predicate: Fn(char) -> bool,
    {
        let mut result = String::new();
        while let Some(&character) = self.input_characters.peek() {
            if predicate(character) {
                result.push(character);
                self.input_characters.next();
                self.advance_position(character);
            } else {
                break;
            }
        }
        result
    }

    fn skip_whitespace(&mut self) {
        while let Some(&character) = self.input_characters.peek() {
            if character.is_whitespace() {
                self.input_characters.next();
                self.advance_position(character);
            } else {
                break;
            }
        }
    }

    pub(crate) fn next_token(&mut self) -> Result<Option<Token>, TokenizeError> {
        if let Some(token) = self.cached_next_token.take() {
            return Ok(Some(token));
        } else {
            self.read_next_token()
        }
    }

    fn read_next_token(&mut self) -> Result<Option<Token>, TokenizeError> {
        self.skip_whitespace();

        if let Some(&character) = self.input_characters.peek() {
            let start_position = self.current_position;
            let token = match character {
                '(' => {
                    self.input_characters.next();
                    self.advance_position(character);
                    Ok(TokenKind::LeftParenthesis)
                }
                ')' => {
                    self.input_characters.next();
                    self.advance_position(character);
                    Ok(TokenKind::RightParenthesis)
                }
                '{' => {
                    self.input_characters.next();
                    self.advance_position(character);
                    Ok(TokenKind::LeftBrace)
                }
                '}' => {
                    self.input_characters.next();
                    self.advance_position(character);
                    Ok(TokenKind::RightBrace)
                }
                '[' => {
                    self.input_characters.next();
                    self.advance_position(character);
                    Ok(TokenKind::LeftBracket)
                }
                ']' => {
                    self.input_characters.next();
                    self.advance_position(character);
                    Ok(TokenKind::RightBracket)
                }
                '.' => {
                    self.input_characters.next();
                    self.advance_position(character);
                    Ok(TokenKind::Dot)
                }
                ':' => {
                    self.input_characters.next();
                    self.advance_position(character);
                    Ok(TokenKind::Colon)
                }
                ',' => {
                    self.input_characters.next();
                    self.advance_position(character);
                    Ok(TokenKind::Comma)
                }
                '"' => self.parse_string_literal(),
                '0'..='9' => self.parse_number_literal(),
                character if character.is_alphanumeric() || character == '_' => {
                    Ok(self.parse_alphanumeric_identifier())
                }
                character if !character.is_whitespace() => Ok(self.parse_operator()),
                _ => Err(TokenizeError::UnexpectedCharacter(character)),
            }?;

            Ok(Some(Token {
                kind: token,
                span: Span {
                    start: start_position,
                    end: self.current_position,
                },
            }))
        } else {
            Ok(None)
        }
    }

    fn parse_alphanumeric_identifier(&mut self) -> TokenKind {
        let identifier =
            self.consume_while(|character| character.is_alphanumeric() || character == '_');
        TokenKind::Identifier(identifier)
    }

    fn parse_operator(&mut self) -> TokenKind {
        let identifier = self.consume_while(|character| {
            !character.is_whitespace()
                && !['(', ')', '.', ':', ',', '{', '}', '[', ']'].contains(&character)
        });
        TokenKind::Operator(identifier)
    }

    fn parse_string_literal(&mut self) -> Result<TokenKind, TokenizeError> {
        // Consume the opening quote
        self.input_characters.next();
        self.advance_position('"');

        let mut string_content = String::new();
        let mut is_escaped = false;

        while let Some(character) = self.input_characters.next() {
            self.advance_position(character);
            match (is_escaped, character) {
                (true, 'n') => {
                    string_content.push('\n');
                    is_escaped = false;
                }
                (true, 't') => {
                    string_content.push('\t');
                    is_escaped = false;
                }
                (true, '\\') => {
                    string_content.push('\\');
                    is_escaped = false;
                }
                (true, '"') => {
                    string_content.push('"');
                    is_escaped = false;
                }
                (false, '\\') => {
                    is_escaped = true;
                }
                (false, '"') => {
                    return Ok(TokenKind::StringLiteral(string_content));
                }
                (false, character) => {
                    string_content.push(character);
                }
                (true, character) => {
                    return Err(TokenizeError::InvalidEscapeSequence(character));
                }
            }
        }
        Err(TokenizeError::UnterminatedStringLiteral)
    }

    fn parse_number_literal(&mut self) -> Result<TokenKind, TokenizeError> {
        let mut number_string = String::new();
        let mut has_decimal_point = false;

        while let Some(&character) = self.input_characters.peek() {
            match character {
                '0'..='9' => {
                    number_string.push(character);
                    self.input_characters.next();
                    self.advance_position(character);
                }
                '.' if !has_decimal_point => {
                    has_decimal_point = true;
                    number_string.push(character);
                    self.input_characters.next();
                    self.advance_position(character);
                }
                '.' => return Err(TokenizeError::InvalidNumberFormatMultipleDecimalPoints),
                _ => break,
            }
        }

        number_string
            .parse::<f64>()
            .map(TokenKind::NumberLiteral)
            .map_err(|error| TokenizeError::FailedToParseNumber(format!("{}", error)))
    }

    pub(crate) fn peek_token(&mut self) -> Result<Option<Token>, TokenizeError> {
        if let Some(token) = self.cached_next_token.as_ref() {
            return Ok(Some(token.clone()));
        } else if let Some(token) = self.read_next_token()? {
            self.cached_next_token = Some(token.clone());
            Ok(Some(token))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer() {
        let input_text = r#"identifier123 "string literal" 123.456 [] {} ()"#;
        let mut tokenizer = Tokenizer::new(input_text);

        let tokens: Result<Vec<Token>, _> =
            std::iter::from_fn(move || tokenizer.next_token().transpose()).collect();

        let mut tokens = tokens.unwrap();

        assert_eq!(tokens.len(), 9);
        tokens.reverse();
        assert_eq!(
            tokens.pop().unwrap().kind,
            TokenKind::Identifier("identifier123".to_string())
        );

        assert_eq!(
            tokens.pop().unwrap().kind,
            TokenKind::StringLiteral("string literal".to_string())
        );

        assert_eq!(
            tokens.pop().unwrap().kind,
            TokenKind::NumberLiteral(123.456)
        );
    }
}
