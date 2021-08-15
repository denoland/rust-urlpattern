// Copyright 2018-2021 the Deno authors. All rights reserved. MIT license.

mod canonicalize_and_process;
mod component;
mod constructor_parser;
mod parser;
mod tokenizer;

use derive_more::Display;

/// An error that occured during parsing.
#[derive(Debug, Display)]
pub enum ParseError {
  Tokenize,
  Url(url::ParseError),
  RegEx(regex::Error),
  SomeRandomOtherError, // TODO: remove
}

impl std::error::Error for ParseError {}

/// The structured input used to create a URL pattern.
pub struct UrlPatternInit {
  protocol: Option<String>,
  username: Option<String>,
  password: Option<String>,
  hostname: Option<String>,
  port: Option<String>,
  pathname: Option<String>,
  search: Option<String>,
  hash: Option<String>,
  base_url: Option<String>,
}

impl UrlPatternInit {
  // Ref: https://wicg.github.io/urlpattern/#process-a-urlpatterninit
  #[allow(clippy::too_many_arguments)]
  fn process(
    &self,
    kind: Option<canonicalize_and_process::ProcessType>,
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

    if let Some(base_url) = &self.base_url {
      let base_url = url::Url::parse(base_url).map_err(ParseError::Url)?;

      // TODO: check if these are correct
      result.protocol = Some(base_url.scheme().to_string());
      result.username = Some(base_url.username().to_string()); // TODO: if empty, none
      result.password = base_url.password().map(String::from);
      result.hostname = Some(url::quirks::hostname(&base_url).to_string());
      todo!("port");
      todo!("pathname");
      result.search = Some(base_url.query().unwrap_or("").to_string());
      result.hash = Some(base_url.fragment().unwrap_or("").to_string());
    }

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
    if let Some(_port) = &self.port {
      todo!()
    }
    if let Some(_pathname) = &self.pathname {
      todo!()
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

/// Input for URLPattern functions.
pub enum URLPatternInput {
  String(String),
  URLPatternInit(UrlPatternInit),
}

// Ref: https://wicg.github.io/urlpattern/#urlpattern
/// A UrlPattern that can be matched against.
pub struct UrlPattern {
  protocol: component::Component,
  username: component::Component,
  password: component::Component,
  hostname: component::Component,
  port: component::Component,
  pathname: component::Component,
  search: component::Component,
  hash: component::Component,
}

impl UrlPattern {
  /// Parse a [UrlPatternInit] and optionally a base url into a [UrlPattern].
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

    let processed_init = init.process(
      Some(canonicalize_and_process::ProcessType::Pattern),
      None,
      None,
      None,
      None,
      None,
      None,
      None,
      None,
    )?;

    todo!();
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

  /// Test if a given input string (with optional base url), matches the pattern.
  pub fn test(&self, _input: &str, _base_url: Option<&str>) -> bool {
    false
  }
}
