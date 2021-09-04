use derive_more::Display;

/// An error that occured during parsing.
#[derive(Debug, Display)]
pub enum ParseError {
  Tokenize,
  Url(url::ParseError),
  RegEx(regex::Error),
  DuplicateName,
  SomeRandomOtherError, // TODO: remove
}

impl std::error::Error for ParseError {}
