# Recursive Link Crawler

This program scrapes a website for all links, and then crawls those links for more links, reporting the status of each link.

The possible states are:

```rust
pub enum UrlState {
    Accessible(Url),
    BadStatus(Url, StatusCode),
    ConnectionFailed(Url),
    TimedOut(Url),
    Malformed(String),
}
```

## Quick Start

```
cargo run <url>
```
