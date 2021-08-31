// Copyright 2018-2021 the Deno authors. All rights reserved. MIT license.

mod canonicalize_and_process;
mod component;
mod constructor_parser;
mod parser;
mod tokenizer;

use canonicalize_and_process::is_special_scheme;
use canonicalize_and_process::special_scheme_default_port;
#[doc(hidden)]
pub use component::Component;

use serde::Deserialize;
use serde::Serialize;

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

/// The structured input used to create a URL pattern.
#[derive(Deserialize, Serialize, Clone, Default, Debug)]
pub struct UrlPatternInit {
  pub protocol: Option<String>,
  pub username: Option<String>,
  pub password: Option<String>,
  pub hostname: Option<String>,
  pub port: Option<String>,
  pub pathname: Option<String>,
  pub search: Option<String>,
  pub hash: Option<String>,
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
  ) -> Result<UrlPatternInit, ParseError> {
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
        url::Url::parse(self_base_url).map_err(ParseError::Url)?;

      // TODO: check if these are correct
      result.protocol = Some(parsed_base_url.scheme().to_string());
      result.username = Some(parsed_base_url.username().to_string());
      result.password =
        Some(parsed_base_url.password().unwrap_or_default().to_string());
      result.hostname =
        Some(parsed_base_url.host_str().unwrap_or_default().to_string());
      result.port =
        Some(parsed_base_url.port().unwrap_or_default().to_string()); // TODO: port_or_known_default?
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
        result.protocol.as_ref().map(|s| &**s),
        &kind,
      )?);
    }
    if let Some(pathname) = &self.pathname {
      result.pathname = Some(pathname.clone());

      if let Some(base_url) = base_url {
        if !base_url.cannot_be_a_base() && is_absolute_pathname(pathname, &kind)
        {
          // TODO: Let slash index be the index of the last U+002F (/) code point found in baseURL’s API pathname string, interpreted as a sequence of code points, or null if there are no instances of the code point.
          let slash_index = Some(0);

          if let Some(slash_index) = slash_index {
            // TODO: Let new pathname be the code point substring from indices 0 to slash index inclusive within baseURL ’s API pathname string .
            let new_pathname = "";
            result.pathname =
              Some(format!("{}{}", new_pathname, result.pathname.unwrap()))
          }
        }
      }

      result.pathname = Some(canonicalize_and_process::process_pathname_init(
        &result.pathname.unwrap(),
        result.protocol.as_ref().map(|s| &**s),
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
  if input.chars().next().unwrap() == '/' {
    return true;
  }
  if kind == &canonicalize_and_process::ProcessType::Url {
    return false;
  }
  // TODO: input code point length
  if input.len() < 2 {
    return false;
  }

  let mut chars = input.chars();
  let x = (chars.next().unwrap(), chars.next().unwrap());
  match x {
    ('\\', '/') => return true,
    ('{', '/') => return true,
    _ => {}
  }

  true
}

// TODO: maybe specify baseURL directly in String variant? (baseURL in UrlPatternInit context will error per spec)
/// Input for URLPattern functions.
#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum URLPatternInput {
  String(String),
  URLPatternInit(UrlPatternInit),
}

// Ref: https://wicg.github.io/urlpattern/#urlpattern
/// A UrlPattern that can be matched against.
#[derive(Deserialize, Serialize)]
pub struct UrlPattern {
  #[doc(hidden)]
  pub protocol: Component,
  #[doc(hidden)]
  pub username: Component,
  #[doc(hidden)]
  pub password: Component,
  #[doc(hidden)]
  pub hostname: Component,
  #[doc(hidden)]
  pub port: Component,
  #[doc(hidden)]
  pub pathname: Component,
  #[doc(hidden)]
  pub search: Component,
  #[doc(hidden)]
  pub hash: Component,
}

impl UrlPattern {
  /// Parse a [URLPatternInput] and optionally a base url into a [UrlPattern].
  pub fn parse(
    input: URLPatternInput,
    base_url: Option<String>,
  ) -> Result<UrlPattern, ParseError> {
    let init = match input {
      URLPatternInput::String(input) => {
        let mut init = constructor_parser::parse_constructor_string(&input)?;
        init.base_url = base_url;
        init
      }
      URLPatternInput::URLPatternInit(input) => {
        if base_url.is_some() {
          return Err(ParseError::SomeRandomOtherError); // TODO: proper error
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
      if is_special_scheme(&protocol) {
        let default_port = special_scheme_default_port(&protocol);
        if default_port == processed_init.port.as_deref() {
          processed_init.port = Some(String::new())
        }
      }
    }

    let protocol_str = processed_init.protocol.clone().unwrap();

    let protocol = Component::compile(
      &protocol_str,
      canonicalize_and_process::canonicalize_protocol,
      Default::default(),
    )?;
    let pathname = if protocol.protocol_component_matches_special_scheme() {
      Component::compile(
        &processed_init.pathname.unwrap(),
        canonicalize_and_process::canonicalize_standard_pathname,
        parser::Options::standard_pathname(),
      )?
    } else {
      Component::compile(
        &processed_init.pathname.unwrap(),
        canonicalize_and_process::canonicalize_cannot_be_a_base_url_pathname,
        Default::default(),
      )?
    };

    Ok(UrlPattern {
      protocol,
      username: Component::compile(
        &processed_init.username.unwrap(),
        canonicalize_and_process::canonicalize_username,
        Default::default(),
      )?,
      password: Component::compile(
        &processed_init.password.unwrap(),
        canonicalize_and_process::canonicalize_password,
        Default::default(),
      )?,
      hostname: Component::compile(
        &processed_init.hostname.unwrap(),
        canonicalize_and_process::canonicalize_hostname,
        parser::Options::hostname(),
      )?,
      port: Component::compile(
        &processed_init.port.unwrap(),
        |port| {
          canonicalize_and_process::canonicalize_port(port, Some(&protocol_str))
        },
        Default::default(),
      )?,
      pathname,
      search: Component::compile(
        &processed_init.search.unwrap(),
        canonicalize_and_process::canonicalize_search,
        Default::default(),
      )?,
      hash: Component::compile(
        &processed_init.hash.unwrap(),
        canonicalize_and_process::canonicalize_hash,
        Default::default(),
      )?,
    })
  }

  pub fn protocol(&self) -> &str {
    &self.protocol.pattern_string
  }
  pub fn username(&self) -> &str {
    &self.username.pattern_string
  }
  pub fn password(&self) -> &str {
    &self.password.pattern_string
  }
  pub fn hostname(&self) -> &str {
    &self.hostname.pattern_string
  }
  pub fn port(&self) -> &str {
    &self.port.pattern_string
  }
  pub fn pathname(&self) -> &str {
    &self.pathname.pattern_string
  }
  pub fn search(&self) -> &str {
    &self.search.pattern_string
  }
  pub fn hash(&self) -> &str {
    &self.hash.pattern_string
  }

  // Ref: https://wicg.github.io/urlpattern/#dom-urlpattern-test
  /// Test if a given input [URLPatternInput] (with optional base url), matches the pattern.
  pub fn test(
    &self,
    input: URLPatternInput,
    base_url: Option<&str>,
  ) -> Result<bool, ParseError> {
    self.matches(input, base_url).map(|res| res.is_some())
  }

  // Ref: https://wicg.github.io/urlpattern/#dom-urlpattern-exec
  // TODO: doc
  pub fn exec(
    &self,
    input: URLPatternInput,
    base_url: Option<&str>,
  ) -> Result<Option<URLPatternResult>, ParseError> {
    self.matches(input, base_url)
  }

  // Ref: https://wicg.github.io/urlpattern/#match
  fn matches(
    &self,
    input: URLPatternInput,
    base_url_string: Option<&str>,
  ) -> Result<Option<URLPatternResult>, ParseError> {
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
      URLPatternInput::URLPatternInit(input) => {
        if base_url_string.is_some() {
          return Err(ParseError::SomeRandomOtherError); // TODO: proper error
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
      URLPatternInput::String(input) => {
        let base_url = if let Some(base_url_string) = base_url_string {
          if let Ok(url) = url::Url::parse(base_url_string) {
            inputs.push(URLPatternInput::String(url.to_string())); // TODO: check
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
        port = url.port().unwrap_or_default().to_string(); // TODO: port_or_known_default?
        todo!("pathname");
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

    if protocol_exec_result.is_none()
      || username_exec_result.is_none()
      || password_exec_result.is_none()
      || hostname_exec_result.is_none()
      || port_exec_result.is_none()
      || pathname_exec_result.is_none()
      || search_exec_result.is_none()
      || hash_exec_result.is_none()
    {
      Ok(None)
    } else {
      Ok(Some(URLPatternResult {
        inputs,
        protocol: self
          .protocol
          .create_match_result(protocol.clone(), protocol_exec_result.unwrap()),
        username: self
          .username
          .create_match_result(username.clone(), username_exec_result.unwrap()),
        password: self
          .password
          .create_match_result(password.clone(), password_exec_result.unwrap()),
        hostname: self
          .hostname
          .create_match_result(hostname.clone(), hostname_exec_result.unwrap()),
        port: self
          .port
          .create_match_result(port.clone(), port_exec_result.unwrap()),
        pathname: self
          .pathname
          .create_match_result(pathname.clone(), pathname_exec_result.unwrap()),
        search: self
          .search
          .create_match_result(search.clone(), search_exec_result.unwrap()),
        hash: self
          .hash
          .create_match_result(hash.clone(), hash_exec_result.unwrap()),
      }))
    }
  }
}

// Ref: https://wicg.github.io/urlpattern/#dictdef-urlpatternresult
// TODO: doc
#[derive(Deserialize, Serialize)]
pub struct URLPatternResult {
  #[serde(skip_deserializing)]
  pub inputs: Vec<URLPatternInput>,

  #[serde(default)]
  pub protocol: URLPatternComponentResult,
  #[serde(default)]
  pub username: URLPatternComponentResult,
  #[serde(default)]
  pub password: URLPatternComponentResult,
  #[serde(default)]
  pub hostname: URLPatternComponentResult,
  #[serde(default)]
  pub port: URLPatternComponentResult,
  #[serde(default)]
  pub pathname: URLPatternComponentResult,
  #[serde(default)]
  pub search: URLPatternComponentResult,
  #[serde(default)]
  pub hash: URLPatternComponentResult,
}

// Ref: https://wicg.github.io/urlpattern/#dictdef-urlpatterncomponentresult
// TODO: doc
#[derive(Deserialize, Serialize, Default)]
pub struct URLPatternComponentResult {
  pub input: String,
  pub groups: std::collections::HashMap<String, String>,
}

#[cfg(test)]
mod tests {
  use serde::Deserialize;

  use super::URLPatternInput;
  use super::URLPatternResult;

  #[derive(Deserialize)]
  #[serde(untagged)]
  pub enum ExpectedMatch {
    String(String),
    URLPatternResult(URLPatternResult),
  }

  #[derive(Deserialize)]
  struct TestCase {
    pattern: Vec<URLPatternInput>,
    #[serde(default)]
    inputs: Vec<URLPatternInput>,
    expected_obj: Option<URLPatternInput>,
    expected_match: Option<ExpectedMatch>,
  }

  fn test_case(case: TestCase) {
    println!("case {:?}", case.pattern);

    let input = case.pattern.get(0).unwrap().clone();
    let base_url = case.pattern.get(1).map(|input| match input {
      crate::URLPatternInput::String(str) => str.clone(),
      crate::URLPatternInput::URLPatternInit(_) => unreachable!(),
    });

    let res = super::UrlPattern::parse(input.clone(), base_url);
    let expected_obj = match case.expected_obj {
      Some(URLPatternInput::String(s)) if s == "error" => {
        assert!(res.is_err());
        return;
      }
      Some(URLPatternInput::String(_)) => unreachable!(),
      Some(URLPatternInput::URLPatternInit(init)) => init,
      None => super::UrlPatternInit::default(),
    };
    let pattern = res.unwrap();

    // TODO(lucacasonato): actually implement logic here!
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
