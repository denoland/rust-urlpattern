// Copyright 2018-2021 the Deno authors. All rights reserved. MIT license.

mod canonicalize_and_process;
mod component;
mod constructor_parser;
mod parser;
mod tokenizer;

#[doc(hidden)]
pub use component::Component;

#[cfg(feature = "serde_")]
use serde::Deserialize;
#[cfg(feature = "serde_")]
use serde::Serialize;

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
#[cfg_attr(feature = "serde_", derive(Deserialize, Serialize))]
#[derive(Clone)]
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
      result.username = Some(base_url.username().to_string());
      result.password =
        Some(base_url.password().unwrap_or_default().to_string());
      result.hostname =
        Some(base_url.host_str().unwrap_or_default().to_string());
      result.port = Some(base_url.port().unwrap_or_default().to_string()); // TODO: port_or_known_default?
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

// TODO: maybe specify baseURL directly in String variant? (baseURL in UrlPatternInit context will error per spec)
/// Input for URLPattern functions.
#[cfg_attr(feature = "serde_", derive(Deserialize, Serialize), serde(untagged))]
#[derive(Clone)]
pub enum URLPatternInput {
  String(String),
  URLPatternInit(UrlPatternInit),
}

// Ref: https://wicg.github.io/urlpattern/#urlpattern
/// A UrlPattern that can be matched against.
#[cfg_attr(feature = "serde_", derive(Deserialize, Serialize))]
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

    // TODO: expose parts of url crate?
    //  If processedInit["protocol"] is a special scheme and processedInit["port"] is its corresponding default port
    if processed_init.protocol {
      processed_init.port = Some(String::new());
    }

    let protocol = Component::compile(
      &processed_init.protocol.unwrap(),
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
        canonicalize_and_process::canonicalize_invalid_baseurl_pathname,
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
        canonicalize_and_process::canonicalize_port,
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
          Some(canonicalize_and_process::ProcessType::Url),
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
#[cfg_attr(feature = "serde_", derive(Serialize))]
pub struct URLPatternResult {
  pub inputs: Vec<URLPatternInput>,

  pub protocol: URLPatternComponentResult,
  pub username: URLPatternComponentResult,
  pub password: URLPatternComponentResult,
  pub hostname: URLPatternComponentResult,
  pub port: URLPatternComponentResult,
  pub pathname: URLPatternComponentResult,
  pub search: URLPatternComponentResult,
  pub hash: URLPatternComponentResult,
}

// Ref: https://wicg.github.io/urlpattern/#dictdef-urlpatterncomponentresult
// TODO: doc
#[cfg_attr(feature = "serde_", derive(Serialize))]
pub struct URLPatternComponentResult {
  pub input: String,
  pub groups: std::collections::HashMap<String, String>,
}
