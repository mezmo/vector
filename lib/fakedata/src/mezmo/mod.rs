use chrono::{DateTime, Utc};
use rand::prelude::*;
use rand_distr::num_traits::Float;

pub mod access_log;
pub mod error_log;
pub mod financial;
pub mod metrics;
pub mod sensor;
pub mod syslog;

/// Wrapper around [SliceRandom::choose] method that expects a value to always be
/// produced since we use const data sets that we ensure are always valid.
fn choose<T: Copy>(slice: &[T]) -> T {
    *slice
        .choose(&mut thread_rng())
        .expect("fakedata should always be able to choose a value")
}

/// Wrapper around [SliceRandom::choose_weighted] method that assumes the weights
/// will be [f32] values and since our datasets are const, expects that every sample
/// operation should succeed.
fn choose_weighted<T: Copy>(slice: &[(T, f32)]) -> T {
    slice
        .choose_weighted(&mut thread_rng(), |item| item.1)
        .expect("fakedata should always be able to choose a weighted value")
        .0
}

/// Samples a value from a normal distribution and then casts to the expected
/// type based on the type parameters. Returns an error if the conversion failed.
fn sample_normal<T, D, O>(dist: &D) -> O
where
    T: Float,
    D: Distribution<T>,
    O: num::NumCast,
{
    num::cast::cast(dist.sample(&mut thread_rng()))
        .expect("fakedata should always be able to sample and cast normal distribution values")
}

const DIGIT_CHARS: &[u8] = b"0123456789";

/// Generates a string of random digits for a given length. The function parameter
/// is called with a max size and should return a value within the range [0, usize).
fn gen_digit_string(num_digits: u8) -> String {
    let mut res = String::new();
    for _ in 0..num_digits {
        let idx = thread_rng().gen_range(0..DIGIT_CHARS.len());
        res.push(DIGIT_CHARS[idx] as char);
    }
    res
}

/// Converts any type that implements the `Into<DateTime<Utc>>` trait into an
/// ISO 8601 formatted string.
fn to_iso8601<T: Into<DateTime<Utc>>>(dt: T) -> String {
    format!("{}", dt.into().format("%+"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;
    use std::time::UNIX_EPOCH;

    #[test]
    fn test_gen_digit_string() {
        let res = gen_digit_string(10);
        let re = Regex::new(r"\d{10}").unwrap();
        assert!(re.is_match(&res));
    }

    #[test]
    fn test_gen_digit_string_zero_len() {
        let res = gen_digit_string(0);
        assert!(res.is_empty());
    }

    #[test]
    fn test_to_iso8601() {
        let res = to_iso8601(UNIX_EPOCH);
        assert_eq!(res, "1970-01-01T00:00:00+00:00");
    }
}
