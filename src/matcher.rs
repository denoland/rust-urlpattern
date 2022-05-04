use crate::regexp::RegExp;
use crate::Error;

#[derive(Debug)]
/// A structured representation of a URLPattern matcher, which can be used to
/// match a URL against a pattern quickly.
pub(crate) struct Matcher<R: RegExp> {
  pub prefix: String,
  pub suffix: String,
  pub inner: InnerMatcher<R>,
}

#[derive(Debug)]
pub(crate) enum InnerMatcher<R: RegExp> {
  /// A literal string matcher.
  ///
  /// # Examples
  /// - /
  /// - /foo
  Literal { literal: String },
  /// A matcher that matches all chars, except the substring specified in
  /// `filter` (if it is set).
  ///
  /// # Examples
  /// - *
  /// - /old/*
  /// - /scripts/*.js
  /// - /:slug
  /// - /blog/:id
  /// - /blog/:id.html
  SingleCapture {
    filter: Option<String>,
    allow_empty: bool,
  },
  /// A regexp matcher. This is a bail-out matcher for arbitrary complexity
  /// matchers.
  ///
  /// # Examples
  /// - /foo/:id?
  RegExp { regexp: Result<R, Error> },
}

impl<R: RegExp> Matcher<R> {
  pub(crate) fn literal(literal: String) -> Self {
    Matcher {
      prefix: "".to_string(),
      suffix: "".to_string(),
      inner: InnerMatcher::Literal { literal },
    }
  }

  pub fn matches<'a>(&self, mut input: &'a str) -> Option<Vec<&'a str>> {
    let prefix_len = self.prefix.len();
    let suffix_len = self.suffix.len();
    let input_len = input.len();
    if prefix_len + suffix_len > 0 {
      // The input must be at least as long as the prefix and suffix combined,
      // because these must both be present, and not overlap.
      if input_len < prefix_len + suffix_len {
        return None;
      }
      if !input.starts_with(&self.prefix) {
        return None;
      }
      if !input.ends_with(&self.suffix) {
        return None;
      }
      input = &input[prefix_len..input_len - suffix_len];
    }

    match &self.inner {
      InnerMatcher::Literal { literal } => (input == literal).then(|| vec![]),
      InnerMatcher::SingleCapture {
        filter,
        allow_empty,
      } => {
        if input.is_empty() && !allow_empty {
          return None;
        }
        if let Some(filter) = filter {
          if input.contains(filter) {
            return None;
          }
        }
        Some(vec![input])
      }
      InnerMatcher::RegExp { regexp, .. } => {
        regexp.as_ref().unwrap().matches(input)
      }
    }
  }
}
