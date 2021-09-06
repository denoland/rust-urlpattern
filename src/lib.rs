// Copyright 2018-2021 the Deno authors. All rights reserved. MIT license.

mod canonicalize_and_process;
mod component;
mod constructor_parser;
mod error;
mod parser;
mod tokenizer;

pub use error::Error;

use crate::canonicalize_and_process::is_special_scheme;
use crate::canonicalize_and_process::special_scheme_default_port;
use crate::component::Component;

use serde::Deserialize;
use serde::Serialize;

/// The structured input used to create a URL pattern.
#[derive(Deserialize, Serialize, Clone, Default, Debug, Eq, PartialEq)]
pub struct UrlPatternInit {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub protocol: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub username: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub password: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub hostname: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub port: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub pathname: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub search: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub hash: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "baseURL")]
  pub base_url: Option<String>,
}

impl UrlPatternInit {
  // Ref: https://wicg.github.io/urlpattern/#process-a-urlpatterninit
  // TODO: use UrlPatternInit for arguments?
  #[allow(clippy::too_many_arguments)]
  fn process(
    &self,
    kind: canonicalize_and_process::ProcessType,
    protocol: Option<String>,
    username: Option<String>,
    password: Option<String>,
    hostname: Option<String>,
    port: Option<String>,
    pathname: Option<String>,
    search: Option<String>,
    hash: Option<String>,
  ) -> Result<UrlPatternInit, Error> {
    let mut result = UrlPatternInit {
      protocol,
      username,
      password,
      hostname,
      port,
      pathname,
      search,
      hash,
      base_url: None,
    };

    let base_url = if let Some(self_base_url) = &self.base_url {
      let parsed_base_url =
        url::Url::parse(self_base_url).map_err(Error::Url)?;

      // TODO: check if these are correct
      result.protocol = Some(parsed_base_url.scheme().to_string());
      result.username = Some(parsed_base_url.username().to_string());
      result.password =
        Some(parsed_base_url.password().unwrap_or_default().to_string());
      result.hostname =
        Some(parsed_base_url.host_str().unwrap_or_default().to_string());
      result.port = Some(url::quirks::port(&parsed_base_url).to_string());
      result.pathname =
        Some(url::quirks::pathname(&parsed_base_url).to_string());
      result.search = Some(parsed_base_url.query().unwrap_or("").to_string());
      result.hash = Some(parsed_base_url.fragment().unwrap_or("").to_string());

      Some(parsed_base_url)
    } else {
      None
    };

    if let Some(protocol) = &self.protocol {
      result.protocol = Some(canonicalize_and_process::process_protocol_init(
        protocol, &kind,
      )?);
    }
    if let Some(username) = &self.username {
      result.username = Some(canonicalize_and_process::process_username_init(
        username, &kind,
      )?);
    }
    if let Some(password) = &self.password {
      result.password = Some(canonicalize_and_process::process_password_init(
        password, &kind,
      )?);
    }
    if let Some(hostname) = &self.hostname {
      result.hostname = Some(canonicalize_and_process::process_hostname_init(
        hostname, &kind,
      )?);
    }
    if let Some(port) = &self.port {
      result.port = Some(canonicalize_and_process::process_port_init(
        port,
        result.protocol.as_deref(),
        &kind,
      )?);
    }
    if let Some(pathname) = &self.pathname {
      result.pathname = Some(pathname.clone());

      if let Some(base_url) = base_url {
        if !base_url.cannot_be_a_base()
          && !is_absolute_pathname(pathname, &kind)
        {
          let baseurl_pathname = url::quirks::pathname(&base_url);
          let slash_index = baseurl_pathname.rfind('/');
          if let Some(slash_index) = slash_index {
            let new_pathname = baseurl_pathname[..=slash_index].to_string();
            result.pathname =
              Some(format!("{}{}", new_pathname, result.pathname.unwrap()));
          }
        }
      }

      result.pathname = Some(canonicalize_and_process::process_pathname_init(
        &result.pathname.unwrap(),
        result.protocol.as_deref(),
        &kind,
      )?);
    }
    if let Some(search) = &self.search {
      result.search = Some(canonicalize_and_process::process_search_init(
        search, &kind,
      )?);
    }
    if let Some(hash) = &self.hash {
      result.hash =
        Some(canonicalize_and_process::process_hash_init(hash, &kind)?);
    }
    Ok(result)
  }
}

// Ref: https://wicg.github.io/urlpattern/#is-an-absolute-pathname
fn is_absolute_pathname(
  input: &str,
  kind: &canonicalize_and_process::ProcessType,
) -> bool {
  if input.is_empty() {
    return false;
  }
  if input.starts_with('/') {
    return true;
  }
  if kind == &canonicalize_and_process::ProcessType::Url {
    return false;
  }
  // TODO: input code point length
  if input.len() < 2 {
    return false;
  }

  input.starts_with("\\/") || input.starts_with("{/")
}

// TODO: maybe specify baseURL directly in String variant? (baseURL in UrlPatternInit context will error per spec)
/// Input for URLPattern functions.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum UrlPatternInput {
  String(String),
  UrlPatternInit(UrlPatternInit),
}

// Ref: https://wicg.github.io/urlpattern/#urlpattern
/// A UrlPattern that can be matched against.
#[derive(Debug, Deserialize, Serialize)]
pub struct UrlPattern {
  protocol: Component,
  username: Component,
  password: Component,
  hostname: Component,
  port: Component,
  pathname: Component,
  search: Component,
  hash: Component,
}

impl UrlPattern {
  // Ref: https://wicg.github.io/urlpattern/#dom-urlpattern-urlpattern
  /// Parse a [UrlPatternInput] and optionally a base url into a [UrlPattern].
  pub fn parse(
    input: UrlPatternInput,
    base_url: Option<String>,
  ) -> Result<UrlPattern, Error> {
    let init = match input {
      UrlPatternInput::String(input) => {
        let mut init = constructor_parser::parse_constructor_string(&input)?;
        if base_url.is_none() && init.protocol.is_none() {
          return Err(Error::BaseUrlRequired);
        }
        init.base_url = base_url;
        init
      }
      UrlPatternInput::UrlPatternInit(input) => {
        if base_url.is_some() {
          return Err(Error::NoBaseUrlWithInitObject);
        }
        input
      }
    };

    let mut processed_init = init.process(
      canonicalize_and_process::ProcessType::Pattern,
      None,
      None,
      None,
      None,
      None,
      None,
      None,
      None,
    )?;

    //  If processedInit["protocol"] is a special scheme and processedInit["port"] is its corresponding default port
    if let Some(protocol) = &processed_init.protocol {
      if is_special_scheme(protocol) {
        let default_port = special_scheme_default_port(protocol);
        if default_port == processed_init.port.as_deref() {
          processed_init.port = Some(String::new())
        }
      }
    }

    let protocol = Component::compile(
      processed_init.protocol.as_deref(),
      canonicalize_and_process::canonicalize_protocol,
      Default::default(),
    )?;

    let hostname_is_ipv6 = processed_init
      .hostname
      .as_deref()
      .map(hostname_pattern_is_ipv6_address)
      .unwrap_or(false);

    let hostname = if hostname_is_ipv6 {
      Component::compile(
        processed_init.hostname.as_deref(),
        canonicalize_and_process::canonicalize_ipv6_hostname,
        parser::Options::hostname(),
      )?
    } else {
      println!("hostname");
      Component::compile(
        processed_init.hostname.as_deref(),
        canonicalize_and_process::canonicalize_hostname,
        parser::Options::hostname(),
      )?
    };

    let pathname = if protocol.protocol_component_matches_special_scheme() {
      Component::compile(
        processed_init.pathname.as_deref(),
        canonicalize_and_process::canonicalize_pathname,
        parser::Options::pathname(),
      )?
    } else {
      Component::compile(
        processed_init.pathname.as_deref(),
        canonicalize_and_process::canonicalize_cannot_be_a_base_url_pathname,
        Default::default(),
      )?
    };

    Ok(UrlPattern {
      protocol,
      username: Component::compile(
        processed_init.username.as_deref(),
        canonicalize_and_process::canonicalize_username,
        Default::default(),
      )?,
      password: Component::compile(
        processed_init.password.as_deref(),
        canonicalize_and_process::canonicalize_password,
        Default::default(),
      )?,
      hostname,
      port: Component::compile(
        processed_init.port.as_deref(),
        |port| canonicalize_and_process::canonicalize_port(port, None),
        Default::default(),
      )?,
      pathname,
      search: Component::compile(
        processed_init.search.as_deref(),
        canonicalize_and_process::canonicalize_search,
        Default::default(),
      )?,
      hash: Component::compile(
        processed_init.hash.as_deref(),
        canonicalize_and_process::canonicalize_hash,
        Default::default(),
      )?,
    })
  }

  /// The pattern used to match against the protocol of the URL.
  pub fn protocol(&self) -> &str {
    &self.protocol.pattern_string
  }

  /// The pattern used to match against the username of the URL.
  pub fn username(&self) -> &str {
    &self.username.pattern_string
  }

  /// The pattern used to match against the password of the URL.
  pub fn password(&self) -> &str {
    &self.password.pattern_string
  }

  /// The pattern used to match against the hostname of the URL.
  pub fn hostname(&self) -> &str {
    &self.hostname.pattern_string
  }

  /// The pattern used to match against the port of the URL.
  pub fn port(&self) -> &str {
    &self.port.pattern_string
  }

  /// The pattern used to match against the pathname of the URL.
  pub fn pathname(&self) -> &str {
    &self.pathname.pattern_string
  }

  /// The pattern used to match against the search string of the URL.
  pub fn search(&self) -> &str {
    &self.search.pattern_string
  }

  /// The pattern used to match against the hash fragment of the URL.
  pub fn hash(&self) -> &str {
    &self.hash.pattern_string
  }

  // Ref: https://wicg.github.io/urlpattern/#dom-urlpattern-test
  /// Test if a given [UrlPatternInput] (with optional base url), matches the
  /// pattern.
  pub fn test(
    &self,
    input: UrlPatternInput,
    base_url: Option<&str>,
  ) -> Result<bool, Error> {
    self.matches(input, base_url).map(|res| res.is_some())
  }

  // Ref: https://wicg.github.io/urlpattern/#dom-urlpattern-exec
  /// Execute the pattern against a [UrlPatternInput] (with optional base url),
  /// returning a [UrlPatternResult] if the pattern matches. If the pattern
  /// doesn't match, returns `None`.
  pub fn exec(
    &self,
    input: UrlPatternInput,
    base_url: Option<&str>,
  ) -> Result<Option<UrlPatternResult>, Error> {
    self.matches(input, base_url)
  }

  // Ref: https://wicg.github.io/urlpattern/#match
  fn matches(
    &self,
    input: UrlPatternInput,
    base_url_string: Option<&str>,
  ) -> Result<Option<UrlPatternResult>, Error> {
    let mut protocol = String::new();
    let mut username = String::new();
    let mut password = String::new();
    let mut hostname = String::new();
    let mut port = String::new();
    let mut pathname = String::new();
    let mut search = String::new();
    let mut hash = String::new();
    let mut inputs = vec![input.clone()];
    match input {
      UrlPatternInput::UrlPatternInit(input) => {
        if base_url_string.is_some() {
          return Err(Error::NoBaseUrlWithInitObject);
        }
        if let Ok(apply_result) = input.process(
          canonicalize_and_process::ProcessType::Url,
          Some(protocol),
          Some(username),
          Some(password),
          Some(hostname),
          Some(port),
          Some(pathname),
          Some(search),
          Some(hash),
        ) {
          protocol = apply_result.protocol.unwrap();
          username = apply_result.username.unwrap();
          password = apply_result.password.unwrap();
          hostname = apply_result.hostname.unwrap();
          port = apply_result.port.unwrap();
          pathname = apply_result.pathname.unwrap();
          search = apply_result.search.unwrap();
          hash = apply_result.hash.unwrap();
        } else {
          return Ok(None);
        }
      }
      UrlPatternInput::String(input) => {
        let base_url = if let Some(base_url_string) = base_url_string {
          if let Ok(url) = url::Url::parse(base_url_string) {
            inputs.push(UrlPatternInput::String(url.to_string())); // TODO: check
            Some(url)
          } else {
            return Ok(None);
          }
        } else {
          None
        };
        // TODO: check
        let url = if let Ok(url) = url::Url::options()
          .base_url(base_url.as_ref())
          .parse(&input)
        {
          url
        } else {
          return Ok(None);
        };

        protocol = url.scheme().to_string();
        username = url.username().to_string();
        password = url.password().unwrap_or_default().to_string();
        hostname = url.host_str().unwrap_or_default().to_string();
        port = url::quirks::port(&url).to_string();
        pathname = url::quirks::pathname(&url).to_string();
        search = url.query().unwrap_or_default().to_string();
        hash = url.fragment().unwrap_or_default().to_string();
      }
    }

    let protocol_exec_result = self.protocol.regexp.captures(&protocol);
    let username_exec_result = self.username.regexp.captures(&username);
    let password_exec_result = self.password.regexp.captures(&password);
    let hostname_exec_result = self.hostname.regexp.captures(&hostname);
    let port_exec_result = self.port.regexp.captures(&port);
    let pathname_exec_result = self.pathname.regexp.captures(&pathname);
    let search_exec_result = self.search.regexp.captures(&search);
    let hash_exec_result = self.hash.regexp.captures(&hash);

    match (
      protocol_exec_result,
      username_exec_result,
      password_exec_result,
      hostname_exec_result,
      port_exec_result,
      pathname_exec_result,
      search_exec_result,
      hash_exec_result,
    ) {
      (
        Some(protocol_exec_result),
        Some(username_exec_result),
        Some(password_exec_result),
        Some(hostname_exec_result),
        Some(port_exec_result),
        Some(pathname_exec_result),
        Some(search_exec_result),
        Some(hash_exec_result),
      ) => Ok(Some(UrlPatternResult {
        inputs,
        protocol: self
          .protocol
          .create_match_result(protocol.clone(), protocol_exec_result),
        username: self
          .username
          .create_match_result(username.clone(), username_exec_result),
        password: self
          .password
          .create_match_result(password.clone(), password_exec_result),
        hostname: self
          .hostname
          .create_match_result(hostname.clone(), hostname_exec_result),
        port: self
          .port
          .create_match_result(port.clone(), port_exec_result),
        pathname: self
          .pathname
          .create_match_result(pathname.clone(), pathname_exec_result),
        search: self
          .search
          .create_match_result(search.clone(), search_exec_result),
        hash: self
          .hash
          .create_match_result(hash.clone(), hash_exec_result),
      })),
      _ => Ok(None),
    }
  }
}

// Ref: https://wicg.github.io/urlpattern/#hostname-pattern-is-an-ipv6-address
fn hostname_pattern_is_ipv6_address(input: &str) -> bool {
  // TODO: code point length
  if input.len() < 2 {
    return false;
  }

  input.starts_with('[') || input.starts_with("{[") || input.starts_with("\\[")
}

// Ref: https://wicg.github.io/urlpattern/#dictdef-urlpatternresult
/// A result of a URL pattern match.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct UrlPatternResult {
  pub inputs: Vec<UrlPatternInput>,

  pub protocol: UrlPatternComponentResult,
  pub username: UrlPatternComponentResult,
  pub password: UrlPatternComponentResult,
  pub hostname: UrlPatternComponentResult,
  pub port: UrlPatternComponentResult,
  pub pathname: UrlPatternComponentResult,
  pub search: UrlPatternComponentResult,
  pub hash: UrlPatternComponentResult,
}

// Ref: https://wicg.github.io/urlpattern/#dictdef-urlpatterncomponentresult
/// A result of a URL pattern match on a single component.
#[derive(Debug, Deserialize, Serialize, Default, PartialEq, Eq)]
pub struct UrlPatternComponentResult {
  /// The matched input for this component.
  pub input: String,
  /// The values for all named groups in the pattern.
  pub groups: std::collections::HashMap<String, String>,
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;

  use serde::Deserialize;
  use url::Url;

  use crate::UrlPatternComponentResult;
  use crate::UrlPatternResult;

  use super::UrlPattern;
  use super::UrlPatternInit;
  use super::UrlPatternInput;

  #[derive(Deserialize)]
  #[serde(untagged)]
  #[allow(clippy::large_enum_variant)]
  enum ExpectedMatch {
    String(String),
    MatchResult(MatchResult),
  }

  #[derive(Deserialize)]
  struct TestCase {
    skip: Option<String>,
    pattern: Vec<UrlPatternInput>,
    #[serde(default)]
    inputs: Vec<UrlPatternInput>,
    expected_obj: Option<UrlPatternInput>,
    expected_match: Option<ExpectedMatch>,
    #[serde(default)]
    exactly_empty_components: Vec<String>,
  }

  #[derive(Debug, Deserialize)]
  struct MatchResult {
    inputs: Option<Vec<UrlPatternInput>>,

    protocol: Option<UrlPatternComponentResult>,
    username: Option<UrlPatternComponentResult>,
    password: Option<UrlPatternComponentResult>,
    hostname: Option<UrlPatternComponentResult>,
    port: Option<UrlPatternComponentResult>,
    pathname: Option<UrlPatternComponentResult>,
    search: Option<UrlPatternComponentResult>,
    hash: Option<UrlPatternComponentResult>,
  }

  fn test_case(case: TestCase) {
    let input = case.pattern.get(0).unwrap().clone();
    let mut base_url = case.pattern.get(1).map(|input| match input {
      UrlPatternInput::String(str) => str.clone(),
      UrlPatternInput::UrlPatternInit(_) => unreachable!(),
    });

    println!("\n=====");
    println!(
      "Pattern: {}, {}",
      serde_json::to_string(&input).unwrap(),
      serde_json::to_string(&base_url).unwrap()
    );

    if let Some(reason) = case.skip {
      println!("🟠 Skipping: {}", reason);
      return;
    }

    let res = UrlPattern::parse(input.clone(), base_url.clone());
    let expected_obj = match case.expected_obj {
      Some(UrlPatternInput::String(s)) if s == "error" => {
        assert!(res.is_err());
        println!("✅ Passed");
        return;
      }
      Some(UrlPatternInput::String(_)) => unreachable!(),
      Some(UrlPatternInput::UrlPatternInit(init)) => init,
      None => UrlPatternInit::default(),
    };
    let pattern = res.expect("failed to parse pattern");

    if let UrlPatternInput::UrlPatternInit(UrlPatternInit {
      base_url: Some(url),
      ..
    }) = &input
    {
      base_url = Some(url.clone())
    }

    macro_rules! assert_field {
      ($field:ident) => {{
        let mut expected = expected_obj.$field;
        if expected == None {
          if case
            .exactly_empty_components
            .contains(&stringify!($field).to_owned())
          {
            expected = Some(String::new())
          } else if let UrlPatternInput::UrlPatternInit(UrlPatternInit {
            $field: Some($field),
            ..
          }) = &input
          {
            expected = Some($field.to_owned())
          } else if let Some(base_url) = &base_url {
            let base_url = Url::parse(base_url).unwrap();
            let field = url::quirks::$field(&base_url);
            let field: String = match stringify!($field) {
              "protocol" if !field.is_empty() => {
                field[..field.len() - 1].to_owned()
              }
              "search" | "hash" if !field.is_empty() => field[1..].to_owned(),
              _ => field.to_owned(),
            };
            expected = Some(field)
          } else {
            expected = Some("*".to_owned())
          }
        }

        let expected = expected.unwrap();
        let pattern = &pattern.$field.pattern_string;

        assert_eq!(
          pattern,
          &expected,
          "pattern for {} does not match",
          stringify!($field)
        );
      }};
    }

    assert_field!(protocol);
    assert_field!(username);
    assert_field!(password);
    assert_field!(hostname);
    assert_field!(port);
    assert_field!(pathname);
    assert_field!(search);
    assert_field!(hash);

    let input = case.inputs.get(0).unwrap().clone();
    let base_url = case.inputs.get(1).map(|input| match input {
      crate::UrlPatternInput::String(str) => str.clone(),
      crate::UrlPatternInput::UrlPatternInit(_) => unreachable!(),
    });

    println!(
      "Input: {}, {}",
      serde_json::to_string(&input).unwrap(),
      serde_json::to_string(&base_url).unwrap(),
    );

    let test_res = pattern.test(input.clone(), base_url.as_deref());
    let exec_res = pattern.exec(input, base_url.as_deref());
    if let Some(ExpectedMatch::String(s)) = &case.expected_match {
      if s == "error" {
        assert!(test_res.is_err());
        assert!(exec_res.is_err());
        println!("✅ Passed");
        return;
      }
    };

    let expected_match = case.expected_match.map(|x| match x {
      ExpectedMatch::String(_) => unreachable!(),
      ExpectedMatch::MatchResult(x) => x,
    });

    let test = test_res.unwrap();
    let actual_match = exec_res.unwrap();

    assert_eq!(
      test,
      expected_match.is_some(),
      "pattern.test result is not correct"
    );

    let expected_match = match expected_match {
      Some(x) => x,
      None => {
        assert!(actual_match.is_none(), "expected match to be None");
        println!("✅ Passed");
        return;
      }
    };

    let actual_match = actual_match.expect("expected match to be Some");

    let exactly_empty_components = case.exactly_empty_components;

    macro_rules! convert_result {
      ($component:ident) => {
        expected_match.$component.unwrap_or_else(|| {
          let mut groups = HashMap::new();
          if !exactly_empty_components
            .contains(&stringify!($component).to_owned())
          {
            groups.insert("0".to_owned(), "".to_owned());
          }
          UrlPatternComponentResult {
            input: "".to_owned(),
            groups,
          }
        })
      };
    }

    let expected_result = UrlPatternResult {
      inputs: expected_match
        .inputs
        .unwrap_or_else(|| actual_match.inputs.clone()),
      protocol: convert_result!(protocol),
      username: convert_result!(username),
      password: convert_result!(password),
      hostname: convert_result!(hostname),
      port: convert_result!(port),
      pathname: convert_result!(pathname),
      search: convert_result!(search),
      hash: convert_result!(hash),
    };

    assert_eq!(
      actual_match, expected_result,
      "pattern.exec result is not correct"
    );

    println!("✅ Passed");
  }

  #[test]
  fn test_cases() {
    let testdata = include_str!("./testdata/urlpatterntestdata.json");
    let cases: Vec<TestCase> = serde_json::from_str(testdata).unwrap();
    for case in cases {
      test_case(case);
    }
  }
}
