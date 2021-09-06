use derive_more::Display;

/// An error that occured during parsing.
#[derive(Display)]
pub enum ParseError {
  #[display(fmt = "tokenizer error: {} (at char {})", _0, _1)]
  Tokenize(TokenizeError, usize),

  #[display(fmt = "a relative input without a base URL is not valid")]
  BaseUrlRequired,

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
