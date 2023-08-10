use crate::mezmo::{choose, choose_weighted};
use chrono::Utc;
use fakedata_generator::gen_ipv4;
use rand::{thread_rng, Rng};

const APACHE_ERROR_TIME_FORMAT: &str = "%d/%b/%Y:%T %z";

const LOG_LEVELS: [(&str, f32); 3] = [("INFO", 8.0), ("WARN", 0.5), ("ERROR", 1.5)];

const INFO_MESSAGES: [&str; 16] = [
    "Digest: generating secret for digest authentication ...",
    "Apache/1.3.11 (Unix) mod_perl/1.21 configured -- resuming normal operations",
    "Apache/2.0.46 (Red Hat) DAV/2 configured -- resuming normal operations",
    "SIGHUP received.  Attempting to restart",
    "suEXEC mechanism enabled (wrapper: /usr/local/apache/sbin/suexec)",
    "workerEnv.init() ok /etc/httpd/conf/workers2.properties",
    "k2_init() Found child 6734 in scoreboard slot 6",
    "k2_init() Found child 6734 in scoreboard slot 7",
    "k2_init() Found child 6734 in scoreboard slot 8",
    "k2_init() Found child 6734 in scoreboard slot 9",
    "k2_init() Found child 6734 in scoreboard slot 10",
    "k2_init() Found child 6766 in scoreboard slot 6",
    "k2_init() Found child 6766 in scoreboard slot 7",
    "k2_init() Found child 6766 in scoreboard slot 8",
    "k2_init() Found child 6766 in scoreboard slot 9",
    "k2_init() Found child 6766 in scoreboard slot 10",
];

const WARN_MESSAGES: [&str; 2] = [
    "pid file /opt/CA/BrightStorARCserve/httpd/logs/httpd.pid overwritten -- Unclean shutdown of previous Apache run?",
    "caught SIGTERM, shutting down"
];

const ERROR_MESSAGES: [&str; 8] = [
    "(11)Resource temporarily unavailable: fork: Unable to fork new process",
    "Directory index forbidden by rule: /var/www/html/",
    "Client sent malformed Host header",
    "user test: authentication failure for \"/~dcid/test1\": Password Mismatch",
    "(111)Connection refused: proxy: HTTP: attempt to connect to 127.0.0.1:8484 (localhost) failed",
    "File does not exist: /var/www/html/robots.txt",
    "mod_jk child workerEnv in error state 6",
    "mod_jk child workerEnv in error state 7",
];

const MODULE_TYPES: [&str; 4] = ["core", "user", "video", "settings"];

fn error_message(level: impl AsRef<str>) -> &'static str {
    match level.as_ref() {
        "ERROR" => choose(&ERROR_MESSAGES),
        "WARN" => choose(&WARN_MESSAGES),
        _ => choose(&INFO_MESSAGES),
    }
}

pub fn apache_error_log_line() -> String {
    let level = choose_weighted(&LOG_LEVELS);
    let pid = thread_rng().gen_range(1000..64000);
    let tid = thread_rng().gen_range(1000..64000);
    let ip = gen_ipv4();
    let port = thread_rng().gen_range(6000..7999);
    let module = choose(&MODULE_TYPES);
    format!(
        "[{}] [{module}:{level}] [pid {pid}:tid {tid}] [client {ip}:{port}] {}",
        Utc::now().format(APACHE_ERROR_TIME_FORMAT),
        error_message(level),
    )
}
