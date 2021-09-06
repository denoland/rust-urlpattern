# urlpattern

This crate implements the [`URLPattern` web API][urlpattern] in Rust. We aim to
follow [the specification][spec] as closely as possible.

[urlpattern]: https://github.com/WICG/urlpattern
[spec]: https://wicg.github.io/urlpattern/

## Example

```rust
use urlpattern::UrlPattern;
use urlpattern::UrlPatternInput;
use urlpattern::UrlPatternInit;

fn main() {
    // Create the UrlPattern to match against.
  let init = UrlPatternInit {
    pathname: Some("/users/:id".to_owned()),
    ..Default::default()
  };
  let pattern = UrlPattern::parse(UrlPatternInput::UrlPatternInit(init), None).unwrap();
 
  // Match the pattern against a URL.
  let url = "https://example.com/users/123".to_owned();
  let result = pattern.exec(UrlPatternInput::String(url), None).unwrap().unwrap();
  assert_eq!(result.pathname.groups.get("id").unwrap(), "123");
}
```

## Contributing

We appreciate your help!

The code of conduct from the Deno repository applies here too:
https://github.com/denoland/deno/blob/main/CODE_OF_CONDUCT.md.
