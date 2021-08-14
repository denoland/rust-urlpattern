// Copyright 2018-2021 the Deno authors. All rights reserved. MIT license.

use crate::tokenizer::Token;
use crate::tokenizer::TokenType;
use crate::ParseError;

// Ref: https://wicg.github.io/urlpattern/#full-wildcard-regexp-value
const FULL_WILDCARD_REGEXP_VALUE: &str = ".*";

// Ref: https://wicg.github.io/urlpattern/#options-header
struct Options {
  delimiter_code_point: String, // TODO: It must contain one ASCII code point or the empty string. maybe Option<char>?
  prefix_code_point: String, // TODO: It must contain one ASCII code point or the empty string. maybe Option<char>?
}

impl Options {
  // Ref: https://wicg.github.io/urlpattern/#generate-a-segment-wildcard-regexp
  // TODO: inline?
  fn generate_segment_wildcard_regexp(&self) -> String {
    format!("[^{}]+?", escape_regexp_string(&self.delimiter_code_point))
  }
}

// Ref: https://wicg.github.io/urlpattern/#part-type
enum PartType {
  FixedText,
  Regexp,
  SegmentWildcard,
  FullWildcard,
}

// Ref: https://wicg.github.io/urlpattern/#part-modifier
#[derive(Eq, PartialEq)]
enum PartModifier {
  None,
  Optional,
  ZeroOrMore,
  OneOrMore,
}

// Ref: https://wicg.github.io/urlpattern/#part
struct Part {
  kind: PartType,
  value: String,
  modifier: PartModifier,
  name: String,
  prefix: String,
  suffix: String,
}

impl Part {
  fn new(kind: PartType, value: String, modifier: PartModifier) -> Self {
    Self {
      kind,
      value,
      modifier,
      name: String::new(),
      prefix: String::new(),
      suffix: String::new(),
    }
  }
}

// Ref: https://wicg.github.io/urlpattern/#pattern-parser
struct PatternParser<F>
where
  F: Fn(&str) -> Result<String, ParseError>,
{
  token_list: Vec<Token>,
  encoding_callback: F,
  segment_wildcard_regexp: String,
  part_list: Vec<Part>,
  pending_fixed_value: String,
  index: usize,
  next_numeric_name: usize,
}

impl<F> PatternParser<F>
where
  F: Fn(&str) -> Result<String, ParseError>,
{
  // Ref: https://wicg.github.io/urlpattern/#try-to-consume-a-token
  fn try_consume_token(&mut self, kind: TokenType) -> Option<&Token> {
    assert!(self.index < self.token_list.len());
    let next_token = &self.token_list[self.index];
    if next_token.kind != kind {
      None
    } else {
      self.index += 1;
      Some(next_token)
    }
  }

  // Ref: https://wicg.github.io/urlpattern/#try-to-consume-a-regexp-or-wildcard-token
  // TODO: inline?
  fn try_consume_regexp_or_wildcard_token(
    &mut self,
    name_token: Option<&Token>,
  ) -> Option<&Token> {
    // TODO: do not mut, instead return
    let mut token = self.try_consume_token(TokenType::Regexp);
    if name_token.is_none() && token.is_none() {
      token = self.try_consume_token(TokenType::Asterisk);
    }
    token
  }

  // Ref: https://wicg.github.io/urlpattern/#try-to-consume-a-modifier-token
  // TODO: inline?
  fn try_consume_modifier_token(&mut self) -> Option<&Token> {
    // TODO: use .or()
    let token = self.try_consume_token(TokenType::OtherModifier);
    if token.is_some() {
      token
    } else {
      self.try_consume_token(TokenType::Asterisk)
    }
  }

  // Ref: https://wicg.github.io/urlpattern/#maybe-add-a-part-from-the-pending-fixed-value
  // TODO: inline?
  fn maybe_add_part_from_pending_fixed_value(
    &mut self,
  ) -> Result<(), ParseError> {
    if self.pending_fixed_value.is_empty() {
      return Ok(());
    }
    let encoded_value = (self.encoding_callback)(&self.pending_fixed_value)?;
    self.pending_fixed_value = String::new();
    self.part_list.push(Part::new(
      PartType::FixedText,
      encoded_value,
      PartModifier::None,
    ));

    Ok(())
  }

  // Ref: https://wicg.github.io/urlpattern/#add-a-part
  fn add_part(
    &mut self,
    prefix: &str,
    name_token: Option<&Token>,
    regexp_or_wildcard_token: Option<&Token>,
    suffix: &str,
    modifier_token: Option<&Token>,
  ) -> Result<(), ParseError> {
    let mut modifier = PartModifier::None;
    if let Some(modifier_token) = modifier_token {
      modifier = match modifier_token.value.as_ref() {
        "?" => PartModifier::Optional,
        "*" => PartModifier::ZeroOrMore,
        "+" => PartModifier::OneOrMore,
        _ => unreachable!(),
      };
    }
    if name_token.is_none()
      && regexp_or_wildcard_token.is_none()
      && modifier == PartModifier::None
    {
      self.pending_fixed_value.push_str(prefix);
      return Ok(());
    }
    self.maybe_add_part_from_pending_fixed_value()?;
    if name_token.is_none() && regexp_or_wildcard_token.is_none() {
      assert!(suffix.is_empty());
      if prefix.is_empty() {
        return Ok(());
      }
      let encoded_value = (self.encoding_callback)(prefix)?;
      self.part_list.push(Part::new(
        PartType::FixedText,
        encoded_value,
        modifier,
      ));
      return Ok(());
    }

    let mut regexp_value: &str = if regexp_or_wildcard_token.is_none() {
      &self.segment_wildcard_regexp
    } else if regexp_or_wildcard_token.unwrap().kind == TokenType::Asterisk {
      FULL_WILDCARD_REGEXP_VALUE
    } else {
      &regexp_or_wildcard_token.unwrap().value
    };

    let mut kind = PartType::Regexp;
    if regexp_value == self.segment_wildcard_regexp {
      kind = PartType::SegmentWildcard;
      regexp_value = "";
    } else if regexp_value == FULL_WILDCARD_REGEXP_VALUE {
      kind = PartType::FullWildcard;
      regexp_value = "";
    }

    let mut name = String::new();
    if let Some(name_token) = name_token {
      name = name_token.value.to_owned();
    } else if regexp_or_wildcard_token.is_some() {
      name = self.next_numeric_name.to_string();
      self.next_numeric_name += 1;
    }
    let encoded_prefix = (self.encoding_callback)(prefix)?;
    let encoded_suffix = (self.encoding_callback)(suffix)?;
    self.part_list.push(Part {
      kind,
      value: regexp_value.to_owned(),
      modifier,
      name,
      prefix: encoded_prefix,
      suffix: encoded_suffix,
    });

    Ok(())
  }

  // Ref: https://wicg.github.io/urlpattern/#consume-text
  fn consume_text(&mut self) -> String {
    let mut result = String::new();
    loop {
      let mut token = self.try_consume_token(TokenType::Char);
      if token.is_none() {
        token = self.try_consume_token(TokenType::EscapedChar);
      }
      if token.is_none() {
        break;
      }
      result.push_str(&token.unwrap().value);
    }
    result
  }

  // Ref: https://wicg.github.io/urlpattern/#consume-a-required-token
  fn consume_required_token(
    &mut self,
    kind: TokenType,
  ) -> Result<&Token, ParseError> {
    let result = self.try_consume_token(kind);
    result.ok_or(ParseError::Tokenize) // TODO: better error
  }
}

// Ref: https://wicg.github.io/urlpattern/#parse-a-pattern-string
fn parse_pattern_string<F>(
  input: String,
  options: Options,
  encoding_callback: F,
) -> Result<Vec<Part>, ParseError>
where
  F: Fn(&str) -> Result<String, ParseError>,
{
  let mut parser = PatternParser {
    token_list: crate::tokenizer::tokenize(
      input,
      crate::tokenizer::TokenizePolicy::Strict,
    )?,
    encoding_callback,
    segment_wildcard_regexp: options.generate_segment_wildcard_regexp(),
    part_list: vec![],
    pending_fixed_value: String::new(),
    index: 0,
    next_numeric_name: 0,
  };

  while parser.index < parser.token_list.len() {
    let char_token = parser.try_consume_token(TokenType::Char);
    let mut name_token = parser.try_consume_token(TokenType::Name);
    let mut regexp_or_wildcard_token =
      parser.try_consume_regexp_or_wildcard_token(name_token);
    if name_token.is_some() || regexp_or_wildcard_token.is_some() {
      let mut prefix = "";
      if let Some(char_token) = char_token {
        prefix = &char_token.value;
      }
      if !prefix.is_empty() && prefix != options.prefix_code_point {
        parser.pending_fixed_value.push_str(prefix);
        prefix = "";
      }
      parser.maybe_add_part_from_pending_fixed_value()?;
      let modifier_token = parser.try_consume_modifier_token();
      parser.add_part(
        prefix,
        name_token,
        regexp_or_wildcard_token,
        "",
        modifier_token,
      )?;
      continue;
    }
    let mut fixed_token = char_token;
    if fixed_token.is_none() {
      fixed_token = parser.try_consume_token(TokenType::EscapedChar);
    }
    if let Some(fixed_token) = fixed_token {
      parser.pending_fixed_value.push_str(&fixed_token.value);
      continue;
    }
    let open_token = parser.try_consume_token(TokenType::Open);
    if open_token.is_some() {
      let prefix = parser.consume_text();
      name_token = parser.try_consume_token(TokenType::Name);
      regexp_or_wildcard_token =
        parser.try_consume_regexp_or_wildcard_token(name_token);
      let suffix = parser.consume_text();
      parser.consume_required_token(TokenType::Close)?;
      let modifier_token = parser.try_consume_modifier_token();
      parser.add_part(
        &prefix,
        name_token,
        regexp_or_wildcard_token,
        &suffix,
        modifier_token,
      )?;
    }
    parser.maybe_add_part_from_pending_fixed_value()?;
    parser.consume_required_token(TokenType::End)?;
  }

  Ok(parser.part_list)
}

// Ref: https://wicg.github.io/urlpattern/#escape-a-regexp-string
// TODO: use fold?
fn escape_regexp_string(input: &str) -> String {
  assert!(input.is_ascii());
  let mut result = String::new();
  for char in input.chars() {
    if matches!(
      char,
      '.'
        | '+'
        | '*'
        | '?'
        | '^'
        | '$'
        | '{'
        | '}'
        | '('
        | ')'
        | '['
        | ']'
        | '|'
        | '/'
        | '\\'
    ) {
      result.push('\\');
    }
    result.push(char);
  }
  result
}
