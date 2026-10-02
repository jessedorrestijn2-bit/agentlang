//! Network access. This is the only place that talks to the internet.
//!
//! Safety choices:
//!   - only https:// URLs are accepted
//!   - the domain is extracted strictly, and tricks such as
//!     `https://example.com@evil.com/` are rejected instead of guessed at
//!   - redirects are blocked, because a redirect could silently leave the
//!     domain the program was given permission for
//!   - there is a 10 second timeout

use std::time::Duration;

/// A function that fetches a URL. Tests replace it with a fake so they never
/// need the internet.
pub type FetchFn = fn(&str) -> Result<String, String>;

/// Extracts the domain from an https URL, or explains why the URL is refused.
pub fn host_of(url: &str) -> Result<String, String> {
    let rest = match url.strip_prefix("https://") {
        Some(r) => r,
        None => return Err("only https:// URLs are allowed".to_string()),
    };
    let authority = rest
        .split(|c: char| c == '/' || c == '?' || c == '#')
        .next()
        .unwrap_or("");
    if authority.is_empty() {
        return Err("the URL has no domain".to_string());
    }
    if authority.contains('@') {
        return Err("URLs with a user name or password are not allowed".to_string());
    }
    let host = authority.split(':').next().unwrap_or("").to_lowercase();
    let valid = !host.is_empty()
        && host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-');
    if !valid {
        return Err(format!("'{}' is not a valid domain", host));
    }
    Ok(host)
}

/// The real network call (HTTPS GET), using the `ureq` crate.
pub fn real_fetch(url: &str) -> Result<String, String> {
    let agent = ureq::AgentBuilder::new()
        .redirects(0)
        .timeout(Duration::from_secs(10))
        .build();
    match agent.get(url).call() {
        Ok(response) => {
            let status = response.status();
            if (300..400).contains(&status) {
                return Err(format!(
                    "the server answered with a redirect (HTTP {}); redirects are blocked because they could leave the permitted domain",
                    status
                ));
            }
            response
                .into_string()
                .map_err(|e| format!("could not read the response: {}", e))
        }
        Err(ureq::Error::Status(code, _)) => Err(format!("the server answered with HTTP {}", code)),
        Err(e) => Err(format!("the request failed: {}", e)),
    }
}
