use derive_more::Display;

use crate::tokenizer::TokenType;

/// An error that occured during parsing.
#[derive(Display)]
pub enum ParseError {
  #[display(fmt = "tokenizer error: {} (at char {})", _0, _1)]
  Tokenize(TokenizeError, usize),

  #[display(fmt = "a relative input without a base URL is not valid")]
  BaseUrlRequired,

  #[display(fmt = "parser error: {}", _0)]
  Parser(ParserError),

  Url(url::ParseError),
  RegEx(regex::Error),
  DuplicateName,
  SomeRandomOtherError, // TODO: remove
}

impl std::error::Error for ParseError {}

impl std::fmt::Debug for ParseError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    std::fmt::Display::fmt(self, f)
  }
}

#[derive(Debug, Display)]
pub enum TokenizeError {
  #[display(fmt = "incomplete escape code")]
  IncompleteEscapeCode,
  #[display(fmt = "invalid name; must be at least length 1")]
  InvalidName,
  #[display(fmt = "invalid regex: {}", _0)]
  InvalidRegex(&'static str),
}

#[derive(Debug, Display)]
pub enum ParserError {
  #[display(fmt = "expected token {}, found '{}' of type {}", _0, _2, _1)]
  ExpectedToken(TokenType, TokenType, String),
}
