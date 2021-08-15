// Copyright 2018-2021 the Deno authors. All rights reserved. MIT license.

use crate::ParseError;

// https://wicg.github.io/urlpattern/#canon-encoding-callbacks

// TODO: dont use dummy.test, expose functions in url crate

// Ref: https://wicg.github.io/urlpattern/#canonicalize-a-protocol
pub fn canonicalize_protocol(value: &str) -> Result<String, ParseError> {
  url::Url::parse(&format!("{}://dummy.test", value))
    .map(|url| url.scheme().to_owned())
    .map_err(ParseError::Url)
}

// Ref: https://wicg.github.io/urlpattern/#canonicalize-a-username
pub fn canonicalize_username(value: &str) -> Result<String, ParseError> {
  let mut url = url::Url::parse("http://dummy.test").unwrap();
  url.set_username(value).unwrap();
  Ok(url.username().to_string())
}

// Ref: https://wicg.github.io/urlpattern/#canonicalize-a-password
pub fn canonicalize_password(value: &str) -> Result<String, ParseError> {
  let mut url = url::Url::parse("http://dummy.test").unwrap();
  url.set_password(Some(value)).unwrap(); // TODO: dont unwrap
  Ok(url.password().unwrap().to_string())
}

// Ref: https://wicg.github.io/urlpattern/#canonicalize-a-hostname
pub fn canonicalize_hostname(value: &str) -> Result<String, ParseError> {
  let mut url = url::Url::parse("http://dummy.test").unwrap();
  url::quirks::set_hostname(&mut url, value).unwrap(); // TODO: dont unwrap
  Ok(url::quirks::hostname(&url).to_string())
}

// Ref: https://wicg.github.io/urlpattern/#canonicalize-a-port
pub fn canonicalize_port(_value: &str) -> Result<String, ParseError> {
  todo!()
}

// Ref: https://wicg.github.io/urlpattern/#canonicalize-a-standard-pathname
pub fn canonicalize_standard_pathname(
  _value: &str,
) -> Result<String, ParseError> {
  todo!()
}

// Ref: https://wicg.github.io/urlpattern/#canonicalize-a-cannot-be-a-base-url-pathname
pub fn canonicalize_invalid_baseurl_pathname(
  _value: &str,
) -> Result<String, ParseError> {
  todo!()
}

// Ref: https://wicg.github.io/urlpattern/#canonicalize-a-search
pub fn canonicalize_search(_value: &str) -> Result<String, ParseError> {
  todo!()
}

// Ref: https://wicg.github.io/urlpattern/#canonicalize-a-search
pub fn canonicalize_hash(_value: &str) -> Result<String, ParseError> {
  todo!()
}

#[derive(Eq, PartialEq)]
pub enum ProcessType {
  Pattern,
  Url,
}

// Ref: https://wicg.github.io/urlpattern/#process-protocol-for-init
pub fn process_protocol_init(value: &str, kind: &Option<ProcessType>) -> Result<String, ParseError> {
  let stripped_value = if value.starts_with(':') {
    value.get(1..).unwrap()
  } else {
    value
  };
  if kind == &Some(ProcessType::Pattern) {
    Ok(stripped_value.to_string())
  } else {
    canonicalize_protocol(stripped_value)
  }
}

// Ref: https://wicg.github.io/urlpattern/#process-username-for-init
pub fn process_username_init(value: &str, kind: &Option<ProcessType>) -> Result<String, ParseError> {
  if kind == &Some(ProcessType::Pattern) {
    Ok(value.to_string())
  } else {
    canonicalize_username(value)
  }
}

// Ref: https://wicg.github.io/urlpattern/#process-password-for-init
pub fn process_password_init(value: &str, kind: &Option<ProcessType>) -> Result<String, ParseError> {
  if kind == &Some(ProcessType::Pattern) {
    Ok(value.to_string())
  } else {
    canonicalize_password(value)
  }
}

// Ref: https://wicg.github.io/urlpattern/#process-hostname-for-init
pub fn process_hostname_init(value: &str, kind: &Option<ProcessType>) -> Result<String, ParseError> {
  if kind == &Some(ProcessType::Pattern) {
    Ok(value.to_string())
  } else {
    canonicalize_hostname(value)
  }
}

// Ref: https://wicg.github.io/urlpattern/#process-port-for-init
pub fn process_port_init(_value: &str, _kind: &Option<ProcessType>) -> Result<String, ParseError> {
  todo!()
}

// Ref: https://wicg.github.io/urlpattern/#process-pathname-for-init
pub fn process_pathname_init(_value: &str, _kind: &Option<ProcessType>) -> Result<String, ParseError> {
  todo!()
}

// Ref: https://wicg.github.io/urlpattern/#process-search-for-init
pub fn process_search_init(value: &str, kind: &Option<ProcessType>) -> Result<String, ParseError> {
  let stripped_value = if value.starts_with('?') {
    value.get(1..).unwrap()
  } else {
    value
  };
  if kind == &Some(ProcessType::Pattern) {
    Ok(stripped_value.to_string())
  } else {
    canonicalize_search(stripped_value)
  }
}

// Ref: https://wicg.github.io/urlpattern/#process-hash-for-init
pub fn process_hash_init(value: &str, kind: &Option<ProcessType>) -> Result<String, ParseError> {
  let stripped_value = if value.starts_with('#') {
    value.get(1..).unwrap()
  } else {
    value
  };
  if kind == &Some(ProcessType::Pattern) {
    Ok(stripped_value.to_string())
  } else {
    canonicalize_hash(stripped_value)
  }
}
