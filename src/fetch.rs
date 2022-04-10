extern crate hyper;
extern crate url;

use std::fmt;
use std::io::Read;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

use self::hyper::status::StatusCode;
use self::hyper::Client;
use self::url::{ParseResult, Url, UrlParser};

use crate::parse;

const TIMEOUT: u64 = 10; // seconds

#[derive(Debug, Clone)]
pub enum UrlState {
    Accessible(Url),
    BadStatus(Url, StatusCode),
    ConnectionFailed(Url),
    TimedOut(Url),
    Malformed(String),
}

impl fmt::Display for UrlState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            UrlState::Accessible(ref url) => write!(f, "Accessible: {}", url),
            UrlState::BadStatus(ref url, ref status) => write!(f, "BadStatus: {} {}", url, status),
            UrlState::ConnectionFailed(ref url) => write!(f, "ConnectionFailed: {}", url),
            UrlState::TimedOut(ref url) => write!(f, "TimedOut: {}", url),
            UrlState::Malformed(ref msg) => write!(f, "Malformed: {}", msg),
        }
    }
}

fn build_url(domain: &str, path: &str) -> ParseResult<Url> {
    let base_url_string = format!("http://{}", domain);
    let base_url = Url::parse(&base_url_string).unwrap();

    let mut raw_url_parser = UrlParser::new();
    let url_parser = raw_url_parser.base_url(&base_url);

    url_parser.parse(path)
}

pub fn url_status(doman: &str, path: &str) -> UrlState {
    match build_url(doman, path) {
        Ok(url) => {
            let (tx, rx) = channel();
            let req_tx = tx.clone();
            let u = url.clone();

            thread::spawn(move || {
                let client = Client::new();
                let url_string = url.serialize();
                let resp = client.get(&url_string).send();

                let _ = req_tx.send(match resp {
                    Ok(r) => {
                        if let StatusCode::Ok = r.status {
                            UrlState::Accessible(url)
                        } else {
                            UrlState::BadStatus(url, r.status)
                        }
                    }
                    Err(_) => UrlState::ConnectionFailed(url),
                });
            });
            thread::spawn(move || {
                thread::sleep(Duration::from_secs(TIMEOUT));
                let _ = tx.send(UrlState::TimedOut(u));
            });

            rx.recv().unwrap()
        }
        Err(_) => return UrlState::Malformed(path.to_owned()),
    }
}

pub fn fetch_url(url: &Url) -> String {
    let client = Client::new();
    let url_string = url.serialize();
    let mut resp = client
        .get(&url_string)
        .send()
        .ok()
        .expect("Could not fetch url");

    let mut body = String::new();
    match resp.read_to_string(&mut body) {
        Ok(_) => body,
        Err(_) => String::new(),
    }
}

pub fn fetch_all_urls(url: &Url) -> Vec<String> {
    let html_src = fetch_url(url);
    let dom = parse::parse_html(&html_src);

    parse::get_urls(dom.document)
}
