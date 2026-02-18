use crate::mezmo::{choose, choose_weighted, gen_digit_string, to_iso8601};
use chrono::{Duration, Utc};
use faker_rand::en_us::names::{FirstName, LastName};
use rand::{Rng, thread_rng};
use serde::Serialize;
use std::time::SystemTime;
use uuid::Uuid;

const ACCESS_ACTIONS: [(&str, f32); 5] = [
    ("login", 4.0),
    ("check_balance", 3.0),
    ("add_charge", 1.5),
    ("remove_charge", 1.0),
    ("logout", 0.5),
];

#[derive(Clone, Copy)]
enum EventTypes {
    Access,
    Transaction,
    Bootup,
}

const EVENT_TYPES: [(EventTypes, f32); 3] = [
    (EventTypes::Access, 6.25),
    (EventTypes::Transaction, 2.5),
    (EventTypes::Bootup, 1.25),
];

const TAX_RATES: [(f64, f32); 21] = [
    (0.0, 1.0),
    (0.0290, 0.2),
    (0.04, 1.0),
    (0.0423, 0.2),
    (0.0445, 0.2),
    (0.045, 0.4),
    (0.0475, 0.2),
    (0.05, 0.4),
    (0.0513, 0.2),
    (0.0530, 0.2),
    (0.055, 0.4),
    (0.056, 0.2),
    (0.0575, 0.2),
    (0.0595, 0.2),
    (0.06, 2.0),
    (0.0625, 0.6),
    (0.0635, 0.2),
    (0.065, 0.6),
    (0.0663, 0.2),
    (0.07, 1.4),
    (0.075, 0.2),
];

const UNIX_DEVICE_PREFIX: [&str; 3] = ["sd", "vd", "xvd"];

const LOWER_CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyz";

fn trunc_f64(value: f64, places: u32) -> f64 {
    let n = 10_i32.pow(places) as f64;
    (value * n).trunc() / n
}

pub struct CardType {
    number_len: u8,
    cvv_len: u8,
}

#[non_exhaustive]
pub struct CardTypes;
impl CardTypes {
    pub const VISA: CardType = CardType {
        number_len: 16,
        cvv_len: 3,
    };

    pub const MASTERCARD: CardType = CardType {
        number_len: 16,
        cvv_len: 3,
    };

    pub const AMERICAN_EXPRESS: CardType = CardType {
        number_len: 15,
        cvv_len: 4,
    };
}

#[derive(Debug, Serialize)]
pub struct CreditCard {
    cc_number: String,
    cc_exp: String,
    cc_cvv: String,
    cc_name: String,
    cc_zip: String,
}

impl CreditCard {
    fn gen_credit_card(card_type: CardType) -> CreditCard {
        let cc_exp = thread_rng().gen_range(0..24);
        let cc_exp = Utc::now()
            .checked_add_signed(Duration::days(cc_exp * 30))
            .expect("adding two years to current time should not overflow");
        let cc_exp = format!("{}", cc_exp.format("%m/%y"));
        let cc_number = gen_digit_string(card_type.number_len);
        let cc_cvv = gen_digit_string(card_type.cvv_len);
        let fname: FirstName = thread_rng().sample(rand::distributions::Standard);
        let lname: LastName = thread_rng().sample(rand::distributions::Standard);
        let cc_name = format!("{fname} {lname}");
        let cc_zip = gen_digit_string(5);

        CreditCard {
            cc_number,
            cc_exp,
            cc_cvv,
            cc_name,
            cc_zip,
        }
    }
}

#[derive(Debug, Serialize)]
struct TransactionDetails {
    product_id: String,
    customer_id: String,
    quantity: u8,
    unit_price: f64,
    net_price: f64,
    tax: f64,
    total_price: f64,
    cc: CreditCard,
    result: bool,
    result_reason: String,
}

impl TransactionDetails {
    pub fn gen_transaction() -> Self {
        let cc: CreditCard = CreditCard::gen_credit_card(CardTypes::VISA);
        let product_id = Uuid::new_v4().to_string();
        // Using UUID v3 for the customer_id based on the credit card name means that random records
        // will contain the same customer_id if the name is the same.
        let customer_id = Uuid::new_v3(&Uuid::NAMESPACE_OID, cc.cc_name.as_bytes()).to_string();
        let quantity: u8 = thread_rng().gen_range(1..20);
        let unit_price = trunc_f64(thread_rng().gen_range(0.01..250.0), 2);
        let net_price = trunc_f64(quantity as f64 * unit_price, 2);
        let tax_rate = choose_weighted(&TAX_RATES);
        let tax = trunc_f64(unit_price * tax_rate, 2);
        let total_price = trunc_f64(net_price + tax, 2);
        let result: bool = thread_rng().gen_bool(0.8);
        let result_reason = if result {
            "card_accepted".to_owned()
        } else {
            "card_denied".to_owned()
        };

        TransactionDetails {
            product_id,
            customer_id,
            quantity,
            unit_price,
            net_price,
            tax,
            total_price,
            cc,
            result,
            result_reason,
        }
    }
}

#[derive(Debug, Serialize)]
struct BootupDetails {
    uptime: usize,
    memory: usize,
    cpu: u8,
    disk: u8,
}

impl BootupDetails {
    fn gen_bootup() -> Self {
        let uptime = thread_rng().gen_range(0..100_000);
        let memory = thread_rng().gen_range(0..30000);
        let cpu = thread_rng().gen_range(0..100);
        let disk = thread_rng().gen_range(0..100);

        Self {
            uptime,
            memory,
            cpu,
            disk,
        }
    }
}

#[derive(Debug, Serialize)]
struct AccessDetails {
    name: String,
    user_id: String,
    action: String,
}

impl AccessDetails {
    fn gen_access() -> Self {
        // Uses a UUID v3 for the user_id because it's based on the MD5 hash of the name, which
        // should produce the same UUID if a duplicate name is picked.
        let fname: FirstName = thread_rng().sample(rand::distributions::Standard);
        let lname: LastName = thread_rng().sample(rand::distributions::Standard);
        let name = format!("{fname} {lname}");
        let user_id = Uuid::new_v3(&Uuid::NAMESPACE_OID, name.as_bytes()).to_string();
        let action = choose_weighted(&ACCESS_ACTIONS).to_string();
        Self {
            name,
            user_id,
            action,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "event")]
enum EventDetails {
    #[serde(rename = "transaction")]
    Transaction { transaction: TransactionDetails },

    #[serde(rename = "bootup")]
    Bootup { bootup: BootupDetails },

    #[serde(rename = "access")]
    Access { access: AccessDetails },
}

#[derive(Debug, Clone, Serialize)]
struct Device {
    id: String,
    name: String,
    vrs: String,
    location: (f64, f64),
    status: String,
}

impl Device {
    fn gen_device() -> Self {
        let id = Uuid::new_v4().to_string();
        let name = format!(
            "/dev/{}{}",
            choose(&UNIX_DEVICE_PREFIX),
            choose(LOWER_CHARS) as char
        );
        let vrs = format!(
            "1.{}.{}",
            thread_rng().gen_range(0..9),
            thread_rng().gen_range(0..9)
        );
        let location = (
            trunc_f64(thread_rng().gen_range(-180.0..180.0), 3),
            trunc_f64(thread_rng().gen_range(-180.0..180.0), 3),
        );
        let status = "active".to_owned();
        Self {
            id,
            name,
            vrs,
            location,
            status,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Event {
    datetime: String,

    device: Device,

    buffer: String,

    #[serde(flatten)]
    event: EventDetails,
}

pub struct EventGenerator {
    pos: usize,
    devices: Vec<Device>,
}

impl EventGenerator {
    pub fn new(cnt: usize) -> Self {
        let mut devices = Vec::with_capacity(cnt);
        for _ in 0..cnt {
            devices.push(Device::gen_device());
        }
        Self { pos: 0, devices }
    }

    pub fn gen_event(&mut self) -> Event {
        let cur = self.pos;
        self.pos = (cur + 1) % self.devices.len();

        let datetime = to_iso8601(SystemTime::now());
        let event = match choose_weighted(&EVENT_TYPES) {
            EventTypes::Transaction => EventDetails::Transaction {
                transaction: TransactionDetails::gen_transaction(),
            },
            EventTypes::Access => EventDetails::Access {
                access: AccessDetails::gen_access(),
            },
            EventTypes::Bootup => EventDetails::Bootup {
                bootup: BootupDetails::gen_bootup(),
            },
        };
        let buffer = Uuid::new_v4().to_string();
        let device = self.devices[cur].clone();
        Event {
            datetime,
            device,
            buffer,
            event,
        }
    }
}
