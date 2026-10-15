//! OpenTelemetry exponential histograms, which Vector has no metric type for.
//!
//! Bucket `index` holds values whose magnitude is in (base^index, base^(index+1)], where
//! base = 2^(2^-scale). Negative values have their own buckets, indexed by magnitude, and values
//! within `zero_threshold` of zero are only counted.
//!
//! Reference: <https://opentelemetry.io/docs/specs/otel/metrics/data-model/#exponentialhistogram>

use std::collections::BTreeMap;

use crate::{
    config::log_schema,
    event::{
        KeyString, Value,
        metric::{Bucket, Metric, MetricArbitrary, mezmo::from_f64_or_zero},
    },
};

pub(super) const ORIGINAL_TYPE: &str = "exponential_histogram";

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ExponentialBuckets {
    pub offset: i64,
    pub counts: Vec<u64>,
}

impl ExponentialBuckets {
    pub(super) fn to_value(&self) -> Value {
        Value::Object(BTreeMap::from([
            ("offset".into(), self.offset.into()),
            (
                "bucket_counts".into(),
                Value::Array(self.counts.iter().copied().map(Value::from).collect()),
            ),
        ]))
    }

    fn from_value(value: &Value) -> Option<Self> {
        let buckets = value.as_object()?;
        let Value::Integer(offset) = buckets.get("offset")? else {
            return None;
        };
        let counts = buckets
            .get("bucket_counts")?
            .as_array()?
            .iter()
            .map(|count| match count {
                // `Value::from(u64)` stores the bits as i64, so this reverses it exactly.
                Value::Integer(count) => Some(*count as u64),
                _ => None,
            })
            .collect::<Option<_>>()?;
        Some(ExponentialBuckets {
            offset: *offset,
            counts,
        })
    }

    fn indexed(&self) -> impl DoubleEndedIterator<Item = (i64, u64)> + '_ {
        self.counts
            .iter()
            .enumerate()
            .map(|(position, count)| (self.offset + position as i64, *count))
    }

    /// Merges each run of 2^shift adjacent buckets into one, which is how a histogram moves to a
    /// lower scale. The arithmetic shift floors, so negative indexes merge the same way positive
    /// ones do: indexes -4 and -3 both become -2 one scale down.
    fn downscale(&self, shift: u32) -> Option<Self> {
        if shift == 0 || self.counts.is_empty() {
            return Some(self.clone());
        }
        let shift = shift.min(i64::BITS - 1);
        let last = self
            .offset
            .checked_add(i64::try_from(self.counts.len()).ok()? - 1)?;
        let offset = self.offset >> shift;
        let mut counts = vec![0u64; usize::try_from((last >> shift) - offset).ok()? + 1];
        for (index, count) in self.indexed() {
            let merged = &mut counts[usize::try_from((index >> shift) - offset).ok()?];
            *merged = merged.checked_add(count)?;
        }
        Some(ExponentialBuckets { offset, counts })
    }

    /// `None` when `previous` has observations that this layout has no room for, or more than
    /// this one in any bucket, which means the series was reset.
    fn subtract_counts(&self, previous: &Self) -> Option<Self> {
        let mut counts = self.counts.clone();
        for (index, previous_count) in previous.indexed().filter(|(_, count)| *count > 0) {
            let position = usize::try_from(index.checked_sub(self.offset)?).ok()?;
            let count = counts.get_mut(position)?;
            *count = count.checked_sub(previous_count)?;
        }
        Some(ExponentialBuckets {
            offset: self.offset,
            counts,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExponentialHistogram {
    pub scale: i32,
    pub zero_count: u64,
    pub zero_threshold: f64,
    pub positive: ExponentialBuckets,
    pub negative: ExponentialBuckets,
}

impl ExponentialHistogram {
    /// Reads the native buckets back from a metric converted from an OpenTelemetry exponential
    /// histogram.
    pub fn from_metric(metric: &Metric) -> Option<Self> {
        let arbitrary = metric.arbitrary_value();
        let original_type = arbitrary
            .value
            .get(log_schema().user_metadata_key())?
            .as_object()?
            .get("original_type")?
            .as_str()?;
        if original_type != ORIGINAL_TYPE {
            return None;
        }
        Self::from_arbitrary(arbitrary)
    }

    pub fn from_arbitrary(arbitrary: &MetricArbitrary) -> Option<Self> {
        let fields = &arbitrary.value;
        let Value::Integer(scale) = fields.get("scale")? else {
            return None;
        };
        let Value::Integer(zero_count) = fields.get("zero_count")? else {
            return None;
        };
        let Value::Float(zero_threshold) = fields.get("zero_threshold")? else {
            return None;
        };
        Some(ExponentialHistogram {
            scale: i32::try_from(*scale).ok()?,
            zero_count: *zero_count as u64,
            zero_threshold: zero_threshold.into_inner(),
            positive: ExponentialBuckets::from_value(fields.get("positive")?)?,
            negative: ExponentialBuckets::from_value(fields.get("negative")?)?,
        })
    }

    /// Only the fields `from_arbitrary` reads.
    pub fn to_arbitrary(&self) -> MetricArbitrary {
        let mut arbitrary = MetricArbitrary::default();
        self.insert_into(&mut arbitrary.value);
        arbitrary
    }

    pub(super) fn insert_into(&self, fields: &mut BTreeMap<KeyString, Value>) {
        fields.insert("scale".into(), i64::from(self.scale).into());
        fields.insert("zero_count".into(), self.zero_count.into());
        fields.insert(
            "zero_threshold".into(),
            from_f64_or_zero(self.zero_threshold),
        );
        fields.insert("positive".into(), self.positive.to_value());
        fields.insert("negative".into(), self.negative.to_value());
    }

    /// An SDK lowers the scale of a cumulative histogram when its range grows, so two points of
    /// one series need not share a layout. Both are brought to the lower scale before the
    /// buckets are subtracted. `None` when `previous` has observations this one lacks, which
    /// means the series was reset.
    pub fn checked_sub(&self, previous: &Self) -> Option<Self> {
        let scale = self.scale.min(previous.scale);
        let current = self.downscale(scale)?;
        let previous = previous.downscale(scale)?;
        Some(ExponentialHistogram {
            scale,
            zero_count: current.zero_count.checked_sub(previous.zero_count)?,
            zero_threshold: self.zero_threshold,
            positive: current.positive.subtract_counts(&previous.positive)?,
            negative: current.negative.subtract_counts(&previous.negative)?,
        })
    }

    fn downscale(&self, scale: i32) -> Option<Self> {
        let shift = u32::try_from(i64::from(self.scale) - i64::from(scale)).ok()?;
        Some(ExponentialHistogram {
            scale,
            zero_count: self.zero_count,
            zero_threshold: self.zero_threshold,
            positive: self.positive.downscale(shift)?,
            negative: self.negative.downscale(shift)?,
        })
    }

    /// Lower bound of bucket `index`.
    pub fn boundary(&self, index: i64) -> f64 {
        #[allow(clippy::cast_precision_loss)]
        let exponent = index as f64 * 2f64.powi(-self.scale);
        2f64.powf(exponent)
    }

    /// The value range and count of every populated bucket outside the zero bucket.
    pub fn bucket_ranges(&self) -> impl Iterator<Item = (f64, f64, u64)> + '_ {
        let negative = self
            .negative
            .indexed()
            .map(|(index, count)| (-self.boundary(index + 1), -self.boundary(index), count));
        let positive = self
            .positive
            .indexed()
            .map(|(index, count)| (self.boundary(index), self.boundary(index + 1), count));
        negative.chain(positive).filter(|(_, _, count)| *count > 0)
    }

    /// Vector buckets have only an upper limit, and each starts where the one below it ends.
    /// Exponential buckets do not always start where the one below ends: the first positive
    /// bucket can start far above the zero bucket. An empty bucket is added at each such gap so
    /// a populated bucket never appears to hold values below its range.
    pub fn buckets(&self) -> Vec<Bucket> {
        let mut buckets: Vec<Bucket> = Vec::new();
        let mut add = |lower: f64, upper_limit: f64, count: u64| {
            let gap = buckets.last().is_none_or(|below| below.upper_limit < lower);
            // With a zero threshold of 0 the zero bucket has no width, and a boundary at -0
            // would repeat its upper limit.
            if gap && lower < upper_limit {
                buckets.push(Bucket {
                    upper_limit: lower,
                    count: 0,
                });
            }
            buckets.push(Bucket { upper_limit, count });
        };

        for (index, count) in self.negative.indexed().rev() {
            add(-self.boundary(index + 1), -self.boundary(index), count);
        }
        add(-self.zero_threshold, self.zero_threshold, self.zero_count);
        for (index, count) in self.positive.indexed() {
            add(self.boundary(index), self.boundary(index + 1), count);
        }
        buckets
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn histogram(scale: i32, positive: (i64, &[u64])) -> ExponentialHistogram {
        ExponentialHistogram {
            scale,
            zero_count: 0,
            zero_threshold: 0.0,
            positive: exponential_buckets(positive.0, positive.1),
            negative: ExponentialBuckets::default(),
        }
    }

    #[test]
    fn subtraction_aligns_offsets() {
        let previous = histogram(0, (3, &[1, 2]));
        let current = histogram(0, (1, &[5, 0, 1, 4]));

        assert_eq!(
            current.checked_sub(&previous),
            Some(histogram(0, (1, &[5, 0, 0, 2])))
        );
    }

    #[test]
    fn subtraction_downscales_to_the_lower_scale() {
        // Going from scale 1 to scale 0 halves each index, rounding down: scale 1 indexes 2 and 3
        // become scale 0 index 1, and scale 1 index 4 becomes scale 0 index 2.
        let previous = histogram(1, (2, &[1, 1, 1]));
        let current = histogram(0, (1, &[3, 5]));

        assert_eq!(
            current.checked_sub(&previous),
            Some(histogram(0, (1, &[1, 4])))
        );
    }

    #[test]
    fn subtraction_handles_negative_offsets() {
        // Scale 1 indexes -3 and -2 become scale 0 indexes -2 and -1.
        let previous = histogram(1, (-3, &[1, 1]));
        let current = histogram(0, (-2, &[2, 1]));

        assert_eq!(
            current.checked_sub(&previous),
            Some(histogram(0, (-2, &[1, 0])))
        );
    }

    #[test]
    fn fewer_observations_is_a_reset() {
        let previous = histogram(0, (0, &[2]));

        assert_eq!(histogram(0, (0, &[1])).checked_sub(&previous), None);
        assert_eq!(histogram(0, (1, &[5])).checked_sub(&previous), None);
    }

    fn exponential_buckets(offset: i64, counts: &[u64]) -> ExponentialBuckets {
        ExponentialBuckets {
            offset,
            counts: counts.to_vec(),
        }
    }

    fn buckets_of(layout: &[(f64, u64)]) -> Vec<Bucket> {
        layout
            .iter()
            .map(|&(upper_limit, count)| Bucket { upper_limit, count })
            .collect()
    }

    #[test]
    fn buckets_fill_the_gap_below_the_first_positive_bucket() {
        let buckets = histogram(0, (10, &[3])).buckets();

        assert_eq!(buckets, buckets_of(&[(0.0, 0), (1024.0, 0), (2048.0, 3)]));
    }

    #[test]
    fn buckets_of_negative_values_only() {
        let mut histogram = histogram(0, (0, &[]));
        histogram.negative = exponential_buckets(0, &[2, 3]);

        assert_eq!(
            histogram.buckets(),
            buckets_of(&[(-4.0, 0), (-2.0, 3), (-1.0, 2), (0.0, 0)])
        );
    }

    #[test]
    fn buckets_of_negative_zero_and_positive_values() {
        let mut histogram = histogram(0, (0, &[3]));
        histogram.negative = exponential_buckets(0, &[1]);
        histogram.zero_count = 2;
        histogram.zero_threshold = 0.5;

        assert_eq!(
            histogram.buckets(),
            buckets_of(&[
                (-2.0, 0),
                (-1.0, 1),
                (-0.5, 0),
                (0.5, 2),
                (1.0, 0),
                (2.0, 3)
            ])
        );
    }

    #[test]
    fn touching_buckets_are_not_gap_filled() {
        let mut histogram = histogram(0, (0, &[1, 2]));
        histogram.negative = exponential_buckets(0, &[3]);
        histogram.zero_count = 4;
        histogram.zero_threshold = 1.0;

        assert_eq!(
            histogram.buckets(),
            buckets_of(&[(-2.0, 0), (-1.0, 3), (1.0, 4), (2.0, 1), (4.0, 2)])
        );
    }

    #[test]
    fn buckets_of_zero_count_only() {
        let mut histogram = histogram(0, (0, &[]));
        histogram.zero_count = 5;

        assert_eq!(histogram.buckets(), buckets_of(&[(0.0, 5)]));
    }

    #[test]
    fn buckets_of_positive_buckets_with_negative_offset() {
        let buckets = histogram(0, (-1, &[3])).buckets();

        assert_eq!(buckets, buckets_of(&[(0.0, 0), (0.5, 0), (1.0, 3)]));
    }

    #[test]
    fn arbitrary_round_trips() {
        let mut histogram = histogram(-2, (-4, &[1, u64::MAX]));
        histogram.zero_count = 7;
        histogram.zero_threshold = 0.5;
        histogram.negative = exponential_buckets(2, &[9]);

        assert_eq!(
            ExponentialHistogram::from_arbitrary(&histogram.to_arbitrary()),
            Some(histogram)
        );
    }
}
