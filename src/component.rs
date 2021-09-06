// Copyright 2018-2021 the Deno authors. All rights reserved. MIT license.

use crate::parser::escape_regexp_string;
use crate::parser::Options;
use crate::parser::Part;
use crate::parser::PartModifier;
use crate::parser::PartType;
use crate::parser::FULL_WILDCARD_REGEXP_VALUE;
use crate::Error;

use serde::Deserialize;
use serde::Serialize;

// Ref: https://wicg.github.io/urlpattern/#component
#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Component {
  pub pattern_string: String,
  #[serde(with = "serde_regex")]
  pub regexp: regex::Regex,
  pub group_name_list: Vec<String>,
}

impl Component {
  // Ref: https://wicg.github.io/urlpattern/#compile-a-component
  pub(crate) fn compile<F>(
    input: Option<&str>,
    encoding_callback: F,
    options: Options,
  ) -> Result<Self, Error>
  where
    F: Fn(&str) -> Result<String, Error>,
  {
    let part_list = crate::parser::parse_pattern_string(
      input.unwrap_or("*"),
      &options,
      encoding_callback,
    )?;
    let (regexp_string, name_list) =
      generate_regular_expression_and_name_list(&part_list, &options);
    let regexp = regex::Regex::new(&regexp_string).map_err(Error::RegEx)?;
    let pattern_string = generate_pattern_string(part_list, &options);
    Ok(Component {
      pattern_string,
      regexp,
      group_name_list: name_list,
    })
  }

  // Ref: https://wicg.github.io/urlpattern/#protocol-component-matches-a-special-scheme
  pub(crate) fn protocol_component_matches_special_scheme(&self) -> bool {
    const SPECIAL_SCHEMES: [&str; 6] =
      ["ftp", "file", "http", "https", "ws", "wss"];
    for scheme in SPECIAL_SCHEMES {
      if self.regexp.captures(scheme).is_some() {
        return true;
      }
    }
    false
  }

  // Ref: https://wicg.github.io/urlpattern/#create-a-component-match-result
  pub(crate) fn create_match_result(
    &self,
    input: String,
    exec_result: regex::Captures,
  ) -> crate::UrlPatternComponentResult {
    let mut iter = exec_result.iter();
    iter.next(); // first match is entire string
    crate::UrlPatternComponentResult {
      input,
      groups: self
        .group_name_list
        .clone()
        .into_iter()
        .zip(iter.map(|e| e.map(|e| e.as_str().to_string())))
        .map(|(name, key)| (name, key.unwrap_or_default()))
        .collect(),
    }
  }
}

// Ref: https://wicg.github.io/urlpattern/#generate-a-regular-expression-and-name-list
fn generate_regular_expression_and_name_list(
  part_list: &[Part],
  options: &Options,
) -> (String, Vec<String>) {
  let mut result = String::from("^");
  let mut name_list = vec![];
  for part in part_list {
    if part.kind == PartType::FixedText {
      if part.modifier == PartModifier::None {
        result.push_str(&escape_regexp_string(&part.value));
      } else {
        result.push_str(&format!(
          "(?:{}){}",
          escape_regexp_string(&part.value),
          part.modifier
        ));
      }
      continue;
    }

    assert!(!part.name.is_empty());
    name_list.push(part.name.clone());
    let regexp_value = if part.kind == PartType::SegmentWildcard {
      options.generate_segment_wildcard_regexp()
    } else if part.kind == PartType::FullWildcard {
      FULL_WILDCARD_REGEXP_VALUE.to_string()
    } else {
      part.value.clone()
    };

    if part.prefix.is_empty() && part.suffix.is_empty() {
      if matches!(part.modifier, PartModifier::None | PartModifier::Optional) {
        result.push_str(&format!("({}){}", regexp_value, part.modifier));
      } else {
        result.push_str(&format!("((?:{}){})", regexp_value, part.modifier));
      }
      continue;
    }
    if matches!(part.modifier, PartModifier::None | PartModifier::Optional) {
      result.push_str(&format!(
        "(?:{}({}){}){}",
        escape_regexp_string(&part.prefix),
        regexp_value,
        escape_regexp_string(&part.suffix),
        part.modifier
      ));
      continue;
    }
    assert!(!part.prefix.is_empty() || !part.suffix.is_empty());
    result.push_str(&format!(
      "(?:{}((?:{})(?:{}{}(?:{}))*){}){}",
      escape_regexp_string(&part.prefix),
      regexp_value,
      escape_regexp_string(&part.suffix),
      escape_regexp_string(&part.prefix),
      regexp_value,
      escape_regexp_string(&part.suffix),
      if part.modifier == PartModifier::ZeroOrMore {
        "?" // TODO: https://github.com/WICG/urlpattern/issues/91
      } else {
        ""
      }
    ));
  }
  result.push('$');
  (result, name_list)
}

// Ref: https://wicg.github.io/urlpattern/#generate-a-pattern-string
fn generate_pattern_string(part_list: Vec<Part>, options: &Options) -> String {
  let mut result = String::new();
  for part in part_list {
    if part.kind == PartType::FixedText {
      if part.modifier == PartModifier::None {
        result.push_str(&escape_pattern_string(&part.value));
        continue;
      }
      result.push_str(&format!(
        "{{{}}}{}",
        escape_pattern_string(&part.value),
        part.modifier
      ));
      continue;
    }
    let needs_grouping = !part.suffix.is_empty()
      || (!part.prefix.is_empty() && part.prefix != options.prefix_code_point);
    assert!(!part.name.is_empty());
    let custom_name = !part.name.chars().next().unwrap().is_ascii_digit();
    if needs_grouping {
      result.push('{');
    }
    result.push_str(&escape_pattern_string(&part.prefix));
    if custom_name {
      result.push(':');
      result.push_str(&part.name);
    }
    match part.kind {
      PartType::FixedText => unreachable!(),
      PartType::Regexp => result.push_str(&format!("({})", part.value)),
      PartType::SegmentWildcard if !custom_name => result
        .push_str(&format!("({})", options.generate_segment_wildcard_regexp())),
      PartType::SegmentWildcard => {}
      PartType::FullWildcard => {
        if custom_name {
          result.push_str(&format!("({})", FULL_WILDCARD_REGEXP_VALUE));
        } else {
          result.push('*');
        }
      }
    }
    result.push_str(&escape_pattern_string(&part.suffix));
    if needs_grouping {
      result.push('}');
    }
    result.push_str(&part.modifier.to_string());
  }
  result
}

// Ref: https://wicg.github.io/urlpattern/#escape-a-pattern-string
fn escape_pattern_string(input: &str) -> String {
  assert!(input.is_ascii());
  let mut result = String::new();
  for char in input.chars() {
    if matches!(char, '+' | '*' | '?' | ':' | '{' | '}' | '(' | ')' | '\\') {
      result.push('\\');
    }
    result.push(char);
  }
  result
}
