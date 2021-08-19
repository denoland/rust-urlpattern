// Copyright 2018-2021 the Deno authors. All rights reserved. MIT license.

// NOTE to all: the code in this crate sometimes slighlty diverges from the
// precise wording of the spec, because rust-url does not expose all the
// routines exactly as the spec wants. The end behaviour should be identical.

use crate::ParseError;

// https://wicg.github.io/urlpattern/#canon-encoding-callbacks

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
  url.set_password(Some(value)).unwrap(); // TODO: dont unwrap, instead ParseError
  Ok(url.password().unwrap().to_string())
}

// Ref: https://wicg.github.io/urlpattern/#canonicalize-a-hostname
pub fn canonicalize_hostname(value: &str) -> Result<String, ParseError> {
  let mut url = url::Url::parse("http://dummy.test").unwrap();
  url::quirks::set_hostname(&mut url, value).unwrap(); // TODO: dont unwrap, instead ParseError
  Ok(url::quirks::hostname(&url).to_string())
}

// Ref: https://wicg.github.io/urlpattern/#canonicalize-a-port
pub fn canonicalize_port(
  value: &str,
  protocol: Option<&str>,
) -> Result<String, ParseError> {
  if let Some(protocol) = protocol {
    let mut url =
      url::Url::parse(&format!("{}://dummy.test", protocol)).unwrap(); // TODO: dont unwrap, instead ParseError
    url::quirks::set_port(&mut url, value).unwrap(); // TODO: dont unwrap, instead ParseError
    Ok(url::quirks::port(&url).to_string())
  } else {
    // If no protocol is given, the url can not have a username/password/port, so
    // we can always return the empty string
    Ok("".to_string())
  }
}

// Ref: https://wicg.github.io/urlpattern/#canonicalize-a-standard-pathname
pub fn canonicalize_standard_pathname(
  value: &str,
) -> Result<String, ParseError> {
  let mut url = url::Url::parse("http://dummy.test").unwrap();
  url.set_path(value);
  Ok(url::quirks::pathname(&url).to_string())
}

// Ref: https://wicg.github.io/urlpattern/#canonicalize-a-cannot-be-a-base-url-pathname
pub fn canonicalize_cannot_be_a_base_url_pathname(
  value: &str,
) -> Result<String, ParseError> {
  let mut url = url::Url::parse("data:dummy,test").unwrap();
  url.set_path(value);
  Ok(url::quirks::pathname(&url).to_string())
}

// Ref: https://wicg.github.io/urlpattern/#canonicalize-a-search
pub fn canonicalize_search(value: &str) -> Result<String, ParseError> {
  let mut url = url::Url::parse("http://dummy.test").unwrap();
  url.set_query(Some(value));
  Ok(url.query().unwrap_or("").to_string())
}

// Ref: https://wicg.github.io/urlpattern/#canonicalize-a-search
pub fn canonicalize_hash(value: &str) -> Result<String, ParseError> {
  let mut url = url::Url::parse("http://dummy.test").unwrap();
  url.set_fragment(Some(value));
  Ok(url.fragment().unwrap_or("").to_string())
}

#[derive(Eq, PartialEq)]
pub enum ProcessType {
  Pattern,
  Url,
}

// Ref: https://wicg.github.io/urlpattern/#process-protocol-for-init
pub fn process_protocol_init(
  value: &str,
  kind: &Option<ProcessType>,
) -> Result<String, ParseError> {
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
pub fn process_username_init(
  value: &str,
  kind: &Option<ProcessType>,
) -> Result<String, ParseError> {
  if kind == &Some(ProcessType::Pattern) {
    Ok(value.to_string())
  } else {
    canonicalize_username(value)
  }
}

// Ref: https://wicg.github.io/urlpattern/#process-password-for-init
pub fn process_password_init(
  value: &str,
  kind: &Option<ProcessType>,
) -> Result<String, ParseError> {
  if kind == &Some(ProcessType::Pattern) {
    Ok(value.to_string())
  } else {
    canonicalize_password(value)
  }
}

// Ref: https://wicg.github.io/urlpattern/#process-hostname-for-init
pub fn process_hostname_init(
  value: &str,
  kind: &Option<ProcessType>,
) -> Result<String, ParseError> {
  if kind == &Some(ProcessType::Pattern) {
    Ok(value.to_string())
  } else {
    canonicalize_hostname(value)
  }
}

// Ref: https://wicg.github.io/urlpattern/#process-port-for-init
pub fn process_port_init(
  port_value: &str,
  protocol_value: Option<&str>,
  kind: &Option<ProcessType>,
) -> Result<String, ParseError> {
  if kind == &Some(ProcessType::Pattern) {
    Ok(port_value.to_string())
  } else {
    canonicalize_port(port_value, protocol_value)
  }
}

// Ref: https://wicg.github.io/urlpattern/#process-pathname-for-init
pub fn process_pathname_init(
  pathname_value: &str,
  protocol_value: Option<&str>,
  kind: &Option<ProcessType>,
) -> Result<String, ParseError> {
  if kind == &Some(ProcessType::Pattern) {
    Ok(pathname_value.to_string())
  } else {
    match protocol_value {
      Some(protocol) if is_special_scheme(protocol) => {
        canonicalize_standard_pathname(pathname_value)
      }
      _ => canonicalize_cannot_be_a_base_url_pathname(pathname_value),
    }
  }
}

// Ref: https://wicg.github.io/urlpattern/#process-search-for-init
pub fn process_search_init(
  value: &str,
  kind: &Option<ProcessType>,
) -> Result<String, ParseError> {
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
pub fn process_hash_init(
  value: &str,
  kind: &Option<ProcessType>,
) -> Result<String, ParseError> {
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

pub fn is_special_scheme(scheme: &str) -> bool {
  matches!(scheme, "http" | "https" | "ws" | "wss" | "ftp" | "file")
}

pub fn special_scheme_default_port(scheme: &str) -> Option<&'static str> {
  match scheme {
    "http" => Some("80"),
    "https" => Some("443"),
    "ws" => Some("80"),
    "wss" => Some("443"),
    "ftp" => Some("21"),
    "file" => None,
    _ => None,
  }
}
