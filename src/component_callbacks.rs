use crate::ParseError;

// https://wicg.github.io/urlpattern/#canon-encoding-callbacks

// TODO: dont use dummy.test, expose functions in url crate

// Ref: https://wicg.github.io/urlpattern/#canonicalize-a-protocol
pub fn canonicalize_protocol(value: &str) -> Result<String, ParseError> {
  url::Url::parse(&format!("{}://dummy.test", value))
    .map(|url| url.scheme().to_owned())
    .map_err(|e| ParseError::Url(e))
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
