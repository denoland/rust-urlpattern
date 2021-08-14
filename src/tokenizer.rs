// Copyright 2018-2021 the Deno authors. All rights reserved. MIT license.

use crate::ParseError;

// Ref: https://wicg.github.io/urlpattern/#tokens
// Ref: https://wicg.github.io/urlpattern/#tokenizing

// Ref: https://wicg.github.io/urlpattern/#token-type
#[derive(Eq, PartialEq)]
pub enum TokenType {
  Open,
  Close,
  Regexp,
  Name,
  Char,
  EscapedChar,
  OtherModifier,
  Asterisk,
  End,
  InvalidChar,
}

// Ref: https://wicg.github.io/urlpattern/#token
pub struct Token {
  pub kind: TokenType,
  index: usize,
  pub value: String,
}

// Ref: https://wicg.github.io/urlpattern/#tokenize-policy
#[derive(Eq, PartialEq)]
pub enum TokenizePolicy {
  Strict,
  Lenient,
}

// Ref: https://wicg.github.io/urlpattern/#tokenizer
struct Tokenizer {
  input: String,
  policy: TokenizePolicy,
  token_list: Vec<Token>,
  index: usize,
  next_index: usize,
  code_point: Option<char>, // TODO: get rid of Option
}

impl Tokenizer {
  // Ref: https://wicg.github.io/urlpattern/#get-the-next-code-point
  // TODO: inline?
  fn get_next_codepoint(&mut self) {
    // TODO: Set tokenizer’s code point to the Unicode code point in tokenizer’s input at the position indicated by tokenizer’s next index.
    //  get Unicode code point
    self.code_point = Some(self.input.chars().nth(self.next_index).unwrap());
    self.next_index += 1;
  }

  // Ref: https://wicg.github.io/urlpattern/#add-a-token-with-default-position-and-length
  // TODO: inline?
  fn add_token_with_default_pos_and_len(&mut self, kind: TokenType) {
    self.add_token_with_default_len(kind, self.next_index, self.index);
  }

  // Ref: https://wicg.github.io/urlpattern/#add-a-token-with-default-length
  // TODO: inline?
  fn add_token_with_default_len(
    &mut self,
    kind: TokenType,
    next_pos: usize,
    value_pos: usize,
  ) {
    self.add_token(kind, next_pos, value_pos, next_pos - value_pos);
  }

  // Ref: https://wicg.github.io/urlpattern/#add-a-token
  // TODO: inline?
  fn add_token(
    &mut self,
    kind: TokenType,
    next_pos: usize,
    value_pos: usize,
    value_len: usize,
  ) {
    self.token_list.push(Token {
      kind,
      index: self.index,
      value: self
        .input
        .get(value_pos..(value_pos + value_len))
        .unwrap()
        .to_owned(), // TODO: check if this is right
    });
    self.index = next_pos;
  }

  // Ref: https://wicg.github.io/urlpattern/#process-a-tokenizing-error
  // TODO: inline?
  fn process_tokenizing_error(
    &mut self,
    next_pos: usize,
    value_pos: usize,
  ) -> Result<(), ParseError> {
    if self.policy == TokenizePolicy::Strict {
      Err(ParseError::Tokenize) // TODO: more descriptive error?
    } else {
      self.add_token_with_default_len(
        TokenType::InvalidChar,
        next_pos,
        value_pos,
      );
      Ok(())
    }
  }

  // Ref: https://wicg.github.io/urlpattern/#seek-and-get-the-next-code-point
  // TODO: inline?
  fn seek_and_get_next_codepoint(&mut self, index: usize) {
    self.next_index = index;
    self.get_next_codepoint();
  }
}

// Ref: https://wicg.github.io/urlpattern/#tokenize
pub fn tokenize(
  input: String,
  policy: TokenizePolicy,
) -> Result<Vec<Token>, ParseError> {
  let mut tokenizer = Tokenizer {
    input,
    policy,
    token_list: vec![],
    index: 0,
    next_index: 0,
    code_point: None,
  };

  while tokenizer.index < tokenizer.input.len() {
    // TODO: https://infra.spec.whatwg.org/#string-code-point-length
    tokenizer.get_next_codepoint();

    // TODO: remove unnecessary continues

    if tokenizer.code_point == Some('*') {
      tokenizer.add_token_with_default_pos_and_len(TokenType::Asterisk);
      continue;
    }
    if matches!(tokenizer.code_point, Some('+') | Some('?')) {
      tokenizer.add_token_with_default_pos_and_len(TokenType::OtherModifier);
      continue;
    }
    if tokenizer.code_point == Some('\\') {
      // TODO: input code point length
      if tokenizer.index == (tokenizer.input.len() - 1) {
        tokenizer
          .process_tokenizing_error(tokenizer.next_index, tokenizer.index)?;
        continue;
      }
      let escaped_index = tokenizer.next_index;
      tokenizer.get_next_codepoint();
      tokenizer.add_token_with_default_len(
        TokenType::EscapedChar,
        tokenizer.next_index,
        escaped_index,
      );
      continue;
    }
    if tokenizer.code_point == Some('{') {
      tokenizer.add_token_with_default_pos_and_len(TokenType::Open);
      continue;
    }
    if tokenizer.code_point == Some('}') {
      tokenizer.add_token_with_default_pos_and_len(TokenType::Close);
      continue;
    }
    if tokenizer.code_point == Some(':') {
      let mut name_pos = tokenizer.next_index;
      let name_start = name_pos;
      // TODO: input code point length
      while name_pos < tokenizer.input.len() {
        tokenizer.seek_and_get_next_codepoint(name_pos);
        let valid_codepoint = is_valid_name_codepoint(
          tokenizer.code_point.unwrap(), // TODO: dont unwrap
          name_pos == name_start,
        );
        if !valid_codepoint {
          break;
        }
        name_pos = tokenizer.next_index;
      }
      if name_pos <= name_start {
        tokenizer.process_tokenizing_error(name_start, tokenizer.index)?;
        continue;
      }
      tokenizer.add_token_with_default_len(
        TokenType::Name,
        name_pos,
        name_start,
      );
    }

    if tokenizer.code_point == Some('(') {
      let mut depth = 1;
      let mut regexp_pos = tokenizer.next_index;
      let regexp_start = regexp_pos;
      let mut error = false;
      // TODO: input code point length
      while regexp_pos < tokenizer.input.len() {
        tokenizer.seek_and_get_next_codepoint(regexp_pos);
        // TODO: dont code_point unwrap
        // TODO: simplify 2 if statements
        if !tokenizer.code_point.unwrap().is_ascii() {
          tokenizer.process_tokenizing_error(regexp_start, tokenizer.index)?;
          error = true;
          break;
        }
        if regexp_pos == regexp_start && tokenizer.code_point == Some('?') {
          tokenizer.process_tokenizing_error(regexp_start, tokenizer.index)?;
          error = true;
          break;
        }
        if tokenizer.code_point == Some('\\') {
          // TODO: input code point length
          if regexp_pos == (tokenizer.input.len() - 1) {
            tokenizer
              .process_tokenizing_error(regexp_start, tokenizer.index)?;
            error = true;
            break;
          }
          tokenizer.get_next_codepoint();
          // TODO: dont code_point unwrap
          if !tokenizer.code_point.unwrap().is_ascii() {
            tokenizer
              .process_tokenizing_error(regexp_start, tokenizer.index)?;
            error = true;
            break;
          }
          regexp_pos = tokenizer.next_index;
          continue;
        }
        if tokenizer.code_point == Some(')') {
          depth -= 1;
          if depth == 0 {
            regexp_pos = tokenizer.next_index;
            break;
          }
        } else if tokenizer.code_point == Some('(') {
          depth += 1;
          // TODO: input code point length
          if regexp_pos == (tokenizer.input.len() - 1) {
            tokenizer
              .process_tokenizing_error(regexp_start, tokenizer.index)?;
            error = true;
            break;
          }
          let temp_pos = tokenizer.next_index;
          tokenizer.get_next_codepoint();
          if tokenizer.code_point != Some('?') {
            tokenizer
              .process_tokenizing_error(regexp_start, tokenizer.index)?;
            error = true;
            break;
          }
          tokenizer.next_index = temp_pos;
        }
        regexp_pos = tokenizer.next_index;
      }
      if error {
        continue;
      }
      if depth != 0 {
        tokenizer.process_tokenizing_error(regexp_start, tokenizer.index)?;
        continue;
      }
      let regexp_len = regexp_pos - regexp_start - 1;
      if regexp_len == 0 {
        tokenizer.process_tokenizing_error(regexp_start, tokenizer.index)?;
        continue;
      }
      tokenizer.add_token(
        TokenType::Regexp,
        regexp_pos,
        regexp_start,
        regexp_len,
      );
    }

    tokenizer.add_token_with_default_pos_and_len(TokenType::Char);
  }

  tokenizer.add_token_with_default_len(
    TokenType::End,
    tokenizer.index,
    tokenizer.index,
  );
  Ok(tokenizer.token_list)
}

// Ref: https://wicg.github.io/urlpattern/#is-a-valid-name-code-point
// TODO: inline?
fn is_valid_name_codepoint(_code_point: char, _first: bool) -> bool {
  todo!()
}