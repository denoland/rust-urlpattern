// Copyright 2018-2021 the Deno authors. All rights reserved. MIT license.

use derive_more::Display;

/// An error that occurred during parsing.
#[derive(Debug, Display)]
pub enum ParseError {}

impl std::error::Error for ParseError {}

/// The structured input used to create a URL pattern.
pub struct UrlPatternInit {
  protocol: String,
  username: String,
  password: String,
  hostname: String,
  port: String,
  pathname: String,
  search: String,
  hash: String,
}

impl UrlPatternInit {
  pub fn parse(
    _pattern: &str,
    _base_url: Option<&str>,
  ) -> Result<UrlPatternInit, ParseError> {
    Ok(UrlPatternInit {
      protocol: "".to_string(),
      username: "".to_string(),
      password: "".to_string(),
      hostname: "".to_string(),
      port: "".to_string(),
      pathname: "".to_string(),
      search: "".to_string(),
      hash: "".to_string(),
    })
  }
}

/// A UrlPattern that can be matched against.
pub struct UrlPattern {}

impl UrlPattern {
  /// Parse a [UrlPatternInit] and optionally a base url into a [UrlPattern].
  pub fn parse(
    _pattern_init: UrlPatternInit,
    _base_url: Option<&str>,
  ) -> Result<UrlPattern, ParseError> {
    Ok(UrlPattern {})
  }

  /// Test if a given input string (with optional base url), matches the pattern.
  pub fn test(&self, _input: &str, _base_url: Option<&str>) -> bool {
    false
  }
}
