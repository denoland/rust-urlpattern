use std::fmt;

use crate::tokenizer::TokenType;

/// A error occurring during URL pattern construction, or matching.
pub enum Error {
  BaseUrlRequired,
  BaseUrlWithInit,
  Tokenizer(TokenizerError, usize),
  Parser(ParserError),
  Url(url::ParseError),
  RegExp(()),
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Error::BaseUrlRequired => {
        f.write_str("a relative input without a base URL is not valid")
      }
      Error::BaseUrlWithInit => f.write_str(
        "specifying both an init object, and a separate base URL is not valid",
      ),
      Error::Tokenizer(err, pos) => {
        write!(f, "tokenizer error: {err} (at char {pos})")
      }
      Error::Parser(err) => write!(f, "parser error: {err}"),
      Error::Url(err) => err.fmt(f),
      Error::RegExp(_) => f.write_str("regexp error"),
    }
  }
}

impl std::error::Error for Error {}

impl std::fmt::Debug for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    std::fmt::Display::fmt(self, f)
  }
}

#[derive(Debug)]
pub enum TokenizerError {
  IncompleteEscapeCode,
  InvalidName,
  InvalidRegex(&'static str),
}

impl fmt::Display for TokenizerError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::IncompleteEscapeCode => f.write_str("incomplete escape code"),
      Self::InvalidName => {
        f.write_str("invalid name; must be at least length 1")
      }
      Self::InvalidRegex(err) => write!(f, "invalid regex: {err}"),
    }
  }
}

#[derive(Debug)]
pub enum ParserError {
  ExpectedToken(TokenType, TokenType, String),
  DuplicateName(String),
}

impl fmt::Display for ParserError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::ExpectedToken(expected_ty, found_ty, found_val) => {
        write!(
          f,
          "expected token {expected_ty:?}, found '{found_val}' of type {found_ty:?}"
        )
      }
      Self::DuplicateName(name) => {
        write!(f, "pattern contains duplicate name {name}")
      }
    }
  }
}
