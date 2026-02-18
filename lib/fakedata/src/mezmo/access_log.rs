use crate::mezmo::{choose, choose_weighted, sample_normal};
use chrono::Utc;
use fakedata_generator::gen_ipv4;
use faker_rand::en_us::internet::{Domain, Username};
use lazy_static::lazy_static;
use rand::{Rng, thread_rng};
use rand_distr::Normal;
use serde::Serialize;

lazy_static! {
    static ref BYTE_SIZE_DIST: Normal<f32> = Normal::new(9000.0, 500.0).unwrap();
}

const APACHE_COMMON_TIME_FORMAT: &str = "%d/%b/%Y:%T %z";
const JSON_TIME_FORMAT: &str = "%d/%b/%Y:%T";

const HTTP_VERSIONS: [(&str, f32); 2] = [("HTTP/1.1", 9.0), ("HTTP/2.0", 1.0)];

const HTTP_METHODS: [(&str, f32); 5] = [
    ("GET", 7.0),
    ("POST", 1.3),
    ("PUT", 1.0),
    ("DELETE", 0.5),
    ("HEAD", 0.2),
];

const ASSET_EXTS: [&str; 4] = [".png", ".gif", ".svg", ".css"];

const API_ENTITIES: [&str; 6] = [
    "customers",
    "accounts",
    "products",
    "messages",
    "notifications",
    "orders",
];

const HTTP_CODES: [(usize, f32); 10] = [
    (200, 5.7),
    (301, 0.3),
    (302, 0.2),
    (400, 0.5),
    (401, 0.5),
    (403, 0.3),
    (404, 1.6),
    (500, 0.4),
    (501, 0.1),
    (503, 0.4),
];

const USER_AGENTS: [(&str, f32); 7] = [
    (
        "Mozilla/5.0 (Linux; Android 5.1.1; SM-G361H Build/LMY48B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/55.0.2883.91 Mobile Safari/537.36",
        2.5,
    ),
    (
        "Mozilla/5.0 (iPhone; CPU iPhone OS 10_3_2 like Mac OS X) AppleWebKit/603.2.4 (KHTML, like Gecko) Version/10.0 Mobile/14F89 Safari/602.1",
        3.0,
    ),
    (
        "Mozilla/5.0 (Linux; Android 8.0.0; SAMSUNG SM-G950F Build/R16NW) AppleWebKit/537.36 (KHTML, like Gecko) SamsungBrowser/8.2 Chrome/63.0.3239.111 Mobile Safari/537.36",
        2.5,
    ),
    (
        "Mozilla/5.0 (Android 7.1.1; Mobile; rv:64.0) Gecko/64.0 Firefox/64.0",
        0.3,
    ),
    (
        "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:15.0) Gecko/20100101 Firefox/15.0.1",
        0.2,
    ),
    (
        "Mozilla/5.0 (X11; CrOS x86_64 8172.45.0) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/51.0.2704.64 Safari/537.36",
        1.0,
    ),
    (
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/42.0.2311.135 Safari/537.36 Edge/12.246",
        0.5,
    ),
];

const BASE_PATH_SEGMENT: [(&str, f32); 6] = [
    ("", 3.0),
    ("js", 1.5),
    ("scripts", 1.5),
    ("api", 1.0),
    ("assets", 1.5),
    ("static", 1.5),
];

fn gen_http_endpoint() -> String {
    let mut path = format!("/{}", choose_weighted(&BASE_PATH_SEGMENT));
    if path == "/" {
        if thread_rng().gen_range(0.0..1.0) < 0.2 {
            path.push_str("favicon.ico");
        }
    } else if path == "/api" {
        path.push('/');
        path.push_str(choose(&API_ENTITIES));
        if thread_rng().gen_bool(0.7) {
            let id = thread_rng().gen_range(127..65535).to_string();
            path.push('/');
            path.push_str(&id);
        }
    } else {
        let ext = if path == "/js/" || path == "/scripts/" {
            ".js"
        } else if path == "/assets/" || path == "/static/" {
            choose(&ASSET_EXTS)
        } else {
            ""
        };
        for _ in 0..31 {
            let ch = thread_rng().gen_range(0..15);
            let ch = char::from_digit(ch, 16).unwrap();
            path.push(ch);
        }
        path.push_str(ext);
    }
    path
}

pub fn apache_common_log_line() -> String {
    // Example log line:
    // 173.159.239.159 - schoen1464 [31/Oct/2020:19:06:10 -0700] "POST /wireless HTTP/2.0" 100 20815
    let byte_size: usize = sample_normal(&*BYTE_SIZE_DIST);
    let version = choose_weighted(&HTTP_VERSIONS);
    let resp_code = choose_weighted(&HTTP_CODES);
    let method = choose_weighted(&HTTP_METHODS);
    format!(
        "{} - - [{}] \"{method} {} {version}\" {resp_code} {byte_size}",
        gen_ipv4(),
        Utc::now().format(APACHE_COMMON_TIME_FORMAT),
        gen_http_endpoint(),
    )
}

pub fn nginx_access_log_line() -> String {
    let user_agent = choose_weighted(&USER_AGENTS);
    let referer: Domain = thread_rng().sample(rand::distributions::Standard);

    // This combined format mirrors what the VRL `parse_nginx_log` is capable of
    // parsing: https://github.com/answerbook/vector/blob/e4b96c57c6c62d91a3d53d1d327cba9501e342f3/lib/vrl/stdlib/src/parse_nginx_log.rs#L81
    // 62.226.10.16 - - [20/Apr/2023:16:50:58 +0000] "POST /js4e5d442adc4ce8033c741dae179a67e HTTP/1.1" 403 8754 "willms.com" "Mozilla/5.0 (Android 7.1.1; Mobile; rv:64.0) Gecko/64.0 Firefox/64.0"
    format!(
        "{} \"{referer}\" \"{user_agent}\"",
        apache_common_log_line()
    )
}

#[derive(Debug, Serialize)]
pub struct JsonAccessLog {
    host: String,
    #[serde(rename = "user-identifier")]
    user_identifier: String,
    datetime: String,
    method: String,
    request: String,
    protocol: String,
    status: usize,
    bytes: u32,
    referer: String,
}

pub fn json_access_log_line() -> JsonAccessLog {
    let bytes = sample_normal(&*BYTE_SIZE_DIST);
    let method = choose_weighted(&HTTP_METHODS).to_string();
    let status = choose_weighted(&HTTP_CODES);
    let protocol = choose_weighted(&HTTP_VERSIONS).to_string();
    let datetime = format!("{}", Utc::now().format(JSON_TIME_FORMAT));
    let user_identifier = thread_rng()
        .sample::<Username, _>(rand::distributions::Standard)
        .to_string();
    let referer = thread_rng()
        .sample::<Domain, _>(rand::distributions::Standard)
        .to_string();
    JsonAccessLog {
        host: gen_ipv4(),
        user_identifier,
        datetime,
        method,
        request: gen_http_endpoint(),
        protocol,
        status,
        bytes,
        referer,
    }
}
