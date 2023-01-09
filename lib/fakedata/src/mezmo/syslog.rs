use crate::mezmo::{choose, choose_weighted};
use chrono::format::{DelayedFormat, StrftimeItems};
use chrono::{Local, SecondsFormat};
use faker_rand::en_us::internet::{Domain, Username};
use lazy_static::lazy_static;
use rand::{distributions::Uniform, prelude::*, thread_rng};

const SYSLOG_3164_FORMAT: &str = "%b %d %T";
const APPLICATION_NAMES: [(&str, f32); 5] = [
    ("sshd", 6.0),
    ("klogind", 3.1),
    ("su", 0.5),
    ("cups", 0.2),
    ("logrotate", 0.2),
];

const SSHD_MESSAGES: [(&str, f32); 9] = [
    ("check pass; user unknown", 2.0),
    ("authentication failure; logname= uid=0 euid=0 tty=NODEVssh ruser= rhost=biblioteka.wsi.edu.pl", 1.0),
    ("authentication failure; logname= uid=0 euid=0 tty=NODEVssh ruser= rhost=218.188.2.4", 1.0),
    ("authentication failure; logname= uid=0 euid=0 tty=NODEVssh ruser= rhost=220-135-151-1.hinet-ip.hinet.net  user=root", 1.0),
    ("authentication failure; logname= uid=0 euid=0 tty=NODEVssh ruser= rhost=061092085098.ctinets.com", 1.0),
    ("authentication failure; logname= uid=0 euid=0 tty=NODEVssh ruser= rhost=d211-116-254-214.rev.krline.net", 1.0), 
    ("authentication failure; logname= uid=0 euid=0 tty=NODEVssh ruser= rhost=62-192-102-94.dsl.easynet.nl  user=root", 1.0),
    ("authentication failure; logname= uid=0 euid=0 tty=NODEVssh ruser= rhost=troi.bluesky-technologies.com  user=root", 1.0),
    ("authentication failure; logname= uid=0 euid=0 tty=NODEVssh ruser= rhost=n219076184117.netvigator.com  user=root", 1.0),
];

const SU_MESSAGES: [&str; 2] = [
    "session opened for user cyrus by (uid=0)",
    "session closed for user cyrus",
];

const KLOGIND_MESSAGES: [&str; 3] = [
    "Authentication failed from 163.27.187.39 (163.27.187.39): Permission denied in replay cache code",
    "Kerberos authentication failed",
    "Authentication failed from 163.27.187.39 (163.27.187.39): Software caused connection abort",
];

const LOGROTATE_MESSAGE: &str = "ALERT exited abnormally with [1]";

const CUPS_MESSAGES: &str = "cupsd shutdown succeeded";

lazy_static! {
    static ref SU_MESSAGES_DIST: Uniform<usize> = Uniform::new(0, SU_MESSAGES.len());
    static ref KLOGIND_MESSAGES_DIST: Uniform<usize> = Uniform::new(0, KLOGIND_MESSAGES.len());
    static ref PRIORITY_DIST: Uniform<usize> = Uniform::new(0, 191);
    static ref PID_DIST: Uniform<usize> = Uniform::new(1, 10_000);
    static ref VERSION_DIST: Uniform<usize> = Uniform::new(1, 4);
}

fn priority() -> usize {
    PRIORITY_DIST.sample(&mut thread_rng())
}

fn pid() -> usize {
    PID_DIST.sample(&mut thread_rng())
}

fn syslog_version() -> usize {
    VERSION_DIST.sample(&mut thread_rng())
}

fn timestamp_syslog_3164() -> DelayedFormat<StrftimeItems<'static>> {
    Local::now().format(SYSLOG_3164_FORMAT)
}

fn timestamp_syslog_5424() -> String {
    Local::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn domain() -> String {
    thread_rng().gen::<Domain>().to_string()
}

fn username() -> String {
    thread_rng().gen::<Username>().to_string()
}

fn error_message(application: &str) -> &'static str {
    match application {
        "logrotate" => LOGROTATE_MESSAGE,
        "cups" => CUPS_MESSAGES,
        "su" => choose(&SU_MESSAGES),
        "klogind" => choose(&KLOGIND_MESSAGES),
        "sshd" => choose_weighted(&SSHD_MESSAGES),
        _ => "error",
    }
}

pub fn syslog_3164_log_line() -> String {
    let app = choose_weighted(&APPLICATION_NAMES);
    format!(
        "<{}>{} {} {}[{}]: {}",
        priority(),
        timestamp_syslog_3164(),
        domain(),
        app,
        pid(),
        error_message(app)
    )
}

pub fn syslog_5424_log_line() -> String {
    // Example log line:
    // <65>2 2020-11-05T18:11:43.975Z chiefubiquitous.io totam 6899 ID44 - Something bad happened
    let app = choose_weighted(&APPLICATION_NAMES);
    format!(
        "<{}>{} {} {} {} {} ID{} - {}",
        priority(),
        syslog_version(),
        timestamp_syslog_5424(),
        domain(),
        username(),
        thread_rng().gen_range(100..9999),
        thread_rng().gen_range(1..999),
        error_message(app)
    )
}
