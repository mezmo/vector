use super::access_log::nginx_access_log_line;
use crate::{logs::syslog_5424_log_line, mezmo::choose_weighted};
use chrono::Utc;
use fakedata_generator::gen_ipv4;
use faker_rand::en_us::internet::Domain;
use rand::{thread_rng, Rng};
use serde::Serialize;

const SPARK_LOG_TIME_FORMAT: &str = "%m/%d/%y %H:%M:%S";
const HDFS_LOG_TIME_FORMAT: &str = "%y%m%d %H%M%S";
const LOG_LEVELS: [(&str, f32); 2] = [("INFO", 8.0), ("WARN", 0.5)];
const LOG_SOURCE_TYPES: [(&str, f32); 4] = [
    ("HDFS", 15.0),
    ("SPARK", 8.0),
    ("NGINX", 0.5),
    ("SYSLOG", 0.5),
];
const APPS: [(&str, f32); 3] = [
    ("task-scheduler", 10.0),
    ("chat-service", 8.0),
    ("file-storage", 1.0),
];

const HDFS_COMPONENTS: [(&str, f32); 3] = [
    ("dfs.DataNodePacketResponder", 10.0),
    ("dfs.FSNamesystem", 6.0),
    ("dfs.DataNodeDataXceiver", 0.5),
];
const HDFS_INFO_MESSAGES: [(&str, f32); 10] = [
    ("PacketResponder 0 for block blk_-6952295868487656571 terminating", 1.0),
    ("BLOCK NameSystem.addStoredBlock: blockMap updated: 10.251.73.220:50010 is added to blk_7128370237687728475 size 67108864", 8.0),
    ("Receiving block blk_5792489080791696128 src: /10.251.30.6:33145 dest: /10.251.30.6:50010", 5.0),
    ("Received block blk_3587508140051953248 of size 67108864 from /10.251.42.84", 1.0),
    ("BLOCK NameSystem.allocateBlock: /user/root/rand/_temporary/_task_200811092030_0001_m_000590_0/part-00590. blk_-1727475099218615100", 3.0),
    ("10.251.194.213:50010 Served block blk_-7724713468912166542 to /10.251.203.80", 0.5),
    ("Verification succeeded for blk_-1547954353065580372", 1.0),
    ("Received block blk_-4411589101766563890 src: /10.250.14.38:37362 dest: /10.250.14.38:50010 of size 67108864", 1.0),
    ("BLOCK ask 10.250.18.114:50010 to delete  blk_-5140072410813878235", 0.5),
    ("Deleting block blk_1781953582842324563 file /mnt/hadoop/dfs/data/current/subdir5/blk_1781953582842324563", 1.0),
];

// From: https://github.com/logpai/loghub-2.0/blob/main/2k_dataset/Spark/Spark_2k.log_structured_corrected.csv
const SPARK_COMPONENTS: [(&str, f32); 3] = [
    ("executor.CoarseGrainedExecutorBackend", 6.0),
    ("spark.SecurityManager", 3.0),
    ("Remoting", 0.5),
];

// From: https://github.com/logpai/loghub-2.0/blob/main/2k_dataset/Spark/Spark_2k.log_structured_corrected.csv
const SPARK_MESSAGES: [(&str, f32); 10] = [
    ("Registered signal handlers for [TERM, HUP, INT]", 1.0),
    ("Changing view acls to: yarn,curi", 0.5),
    ("Changing modify acls to: yarn,curi", 0.5),
    ("SecurityManager: authentication disabled; ui acls disabled; users with view permissions: Set(yarn, curi); users with modify permissions: Set(yarn, curi)", 1.0),
    ("Created local directory at /opt/hdfs/nodemanager/usercache/curi/appcache/application_1485248649253_0147/blockmgr-70293f72-844a-4b39-9ad6-fb0ad7e364e4", 10.0),
    ("Successfully started service 'org.apache.spark.network.netty.NettyBlockTransferService' on port 40984.", 6.0),
    ("Server created on 40984", 0.5),
    ("executor.CoarseGrainedExecutorBackend: Registered signal handlers for [TERM, HUP, INT]", 1.0),
    ("Connecting to driver: spark://CoarseGrainedScheduler@10.10.34.11:48069", 3.0),
    ("Created local directory at /opt/hdfs/nodemanager/usercache/curi/appcache/application_1485248649253_0147/blockmgr-70293f72-844a-4b39-9ad6-fb0ad7e364e4", 2.0),
];

pub fn infra_log_line() -> InfraLog {
    InfraLog::new()
}

#[derive(Debug, Serialize)]
pub struct InfraLog {
    app: String,
    host: String,
    level: String,
    line: String,
}

impl InfraLog {
    fn new() -> Self {
        let app = choose_weighted(&APPS);
        let level = choose_weighted(&LOG_LEVELS);
        let source_type = choose_weighted(&LOG_SOURCE_TYPES);

        Self {
            app: app.to_string(),
            host: thread_rng().gen::<Domain>().to_string(),
            level: level.to_string(),
            line: match source_type {
                "SPARK" => spark_log_line(level),
                "NGINX" => nginx_access_log_line(),
                "SYSLOG" => syslog_5424_log_line(),
                _ => hdfs_log_line(level),
            },
        }
    }
}

macro_rules! hdfs_warning {
    ($server_ip:expr, $block_id:expr, $client_ip:expr) => {
        format!(
            "{}:Got exception while serving blk_{} to /{}:",
            $server_ip, $block_id, $client_ip
        )
    };
}

/**
 * Generates HDFS logs matching the following format:
 * <date:YYMMDD> <time:HHMMSS> <pid> <level> <source_component>: <message>
 */
fn hdfs_log_line(level: &str) -> String {
    let pid = thread_rng().gen_range(1024..65535);
    let message = match level {
        "INFO" => choose_weighted(&HDFS_INFO_MESSAGES).to_string(),
        _ => {
            let server_ip = format!("{}:{}", gen_ipv4(), thread_rng().gen_range(1000..65535));
            let block_id = thread_rng().gen::<u64>();
            let client_ip = gen_ipv4();
            hdfs_warning!(server_ip, block_id, client_ip)
        }
    };

    format!(
        "{} {} {} {}: {}",
        Utc::now().format(HDFS_LOG_TIME_FORMAT),
        pid,
        level,
        choose_weighted(&HDFS_COMPONENTS),
        message
    )
}

/**
 * Generate spark log line matching the following template:
 * date:MM/DD/YY> <time:HH:MM:SS> <level> <component>: <message>
 */
fn spark_log_line(level: &str) -> String {
    format!(
        "{} {} {}: {}",
        Utc::now().format(SPARK_LOG_TIME_FORMAT),
        level,
        choose_weighted(&SPARK_COMPONENTS),
        choose_weighted(&SPARK_MESSAGES)
    )
}
