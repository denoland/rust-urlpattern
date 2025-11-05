#![no_main]
use urlpattern::{UrlPattern, UrlPatternInit, UrlPatternOptions};
use libfuzzer_sys::arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    protocol: Option<String>,
    username: Option<String>,
    password: Option<String>,
    hostname: Option<String>,
    port: Option<String>,
    pathname: Option<String>,
    search: Option<String>,
    hash: Option<String>,
    base_url: Option<String>,
    ignore_case: bool,
}

fuzz_target!(|input: FuzzInput| {
    let init = UrlPatternInit {
        protocol: input.protocol,
        username: input.username,
        password: input.password,
        hostname: input.hostname,
        port: input.port,
        pathname: input.pathname,
        search: input.search,
        hash: input.hash,
        base_url: input.base_url.and_then(|s| s.parse().ok()),
    };
    let options = UrlPatternOptions {
        ignore_case: input.ignore_case,
    };
    let _ = UrlPattern::<regex::Regex>::parse(init, options);
});
