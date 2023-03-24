#![allow(clippy::excessive_precision)]
/// Module for generating fake environmental sensor telemetry data based on a sample
/// of a real source. This is the same data set used in the Mezmo demo systems code base
/// but modified to avoid file I/O from a sample data file and limited memory usage.
///
/// Source:
/// https://www.kaggle.com/datasets/garystafford/environmental-sensor-data-132k?resource=download
use crate::mezmo::{choose_weighted, sample_normal};
use lazy_static::lazy_static;
use rand_distr::Normal;
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

lazy_static! {
    static ref TEMP_DIST: Normal<f32> = Normal::new(22.45398735, 2.698346951).unwrap();
    static ref HUM_DIST: Normal<f32> = Normal::new(60.51169396, 11.36648939).unwrap();
    static ref CO_DIST: Normal<f32> = Normal::new(0.004638844633, 0.001250026283).unwrap();
    static ref SMOKE_DIST: Normal<f32> = Normal::new(0.01926361178, 0.00408613006).unwrap();
    static ref LPG_DIST: Normal<f32> = Normal::new(0.007237125655, 0.001444115679).unwrap();
}

const DEVICE_MAC_ADDRS: [(&str, f32); 3] = [
    ("b8:27:eb:bf:9d:51", 4.6),
    ("00:0f:00:70:91:0a", 2.8),
    ("1c:bf:ce:15:ec:4d", 2.6),
];

const LIGHT_READINGS: [(bool, f32); 2] = [(true, 2.8), (false, 7.2)];

const MOTION_READINGS: [(bool, f32); 2] = [(true, 0.1), (false, 9.9)];

#[derive(Debug, Serialize)]
pub struct SensorData {
    co: f32,
    humidity: f32,
    light: bool,
    lpg: f32,
    motion: bool,
    smoke: f32,
    temp: f32,
}

impl SensorData {
    fn gen_sensor_data() -> Self {
        let co = sample_normal(&*CO_DIST);
        let humidity = sample_normal(&*HUM_DIST);
        let light = choose_weighted(&LIGHT_READINGS);
        let lpg = sample_normal(&*LPG_DIST);
        let motion = choose_weighted(&MOTION_READINGS);
        let smoke = sample_normal(&*SMOKE_DIST);
        let temp = sample_normal(&*TEMP_DIST);
        Self {
            co,
            humidity,
            light,
            lpg,
            motion,
            smoke,
            temp,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SensorMqttMessage {
    ts: f64,
    device_id: String,
    data: SensorData,
}

impl SensorMqttMessage {
    pub fn gen_sensor_message() -> Self {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("since epoch should be positive values")
            .as_secs_f64();
        let device_id = choose_weighted(&DEVICE_MAC_ADDRS);
        let data = SensorData::gen_sensor_data();

        Self {
            ts,
            device_id: device_id.to_string(),
            data,
        }
    }
}
