use std::ops::{Range, RangeInclusive};

pub trait TrackValue: PartialEq {
    type Value: Copy + PartialEq;

    fn get_value(&self, step: Option<Self::Value>, pct: f32) -> Self::Value;
    fn set_value(&self, value: Self::Value) -> f32;
}

macro_rules! impl_track_value_range {
    ($val:ty) => {
        impl TrackValue for Range<$val> {
            type Value = $val;

            fn get_value(&self, step: Option<Self::Value>, pct: f32) -> Self::Value {
                let min = self.start as f32;
                let max = (self.end as f32 - 1.0).max(min);
                let value = min + pct * (max - min);
                step.map_or(value, |s| (value / s as f32).round() * s as f32)
                    .clamp(min, max)
                    .round() as Self::Value
            }

            fn set_value(&self, value: Self::Value) -> f32 {
                let min = self.start as f32;
                let span = (self.end as f32 - min - 1.0).max(1.0);
                (value as f32 - min) / span
            }
        }
    };
}
macro_rules! impl_track_value_range_inclusive_int {
    ($val:ty) => {
        impl TrackValue for RangeInclusive<$val> {
            type Value = $val;

            fn get_value(&self, step: Option<Self::Value>, pct: f32) -> Self::Value {
                let min = *self.start() as f32;
                let max = *self.end() as f32;
                let value = min + pct * (max - min);
                step.map_or(value, |s| (value / s as f32).round() * s as f32)
                    .clamp(min, max)
                    .round() as Self::Value
            }

            fn set_value(&self, value: Self::Value) -> f32 {
                let min = *self.start() as f32;
                let span = (*self.end() as f32 - min).max(1.0);
                (value as f32 - min) / span
            }
        }
    };
}
macro_rules! impl_track_value_range_inclusive_float {
    ($val:ty) => {
        impl TrackValue for RangeInclusive<$val> {
            type Value = $val;

            fn get_value(&self, step: Option<Self::Value>, pct: f32) -> Self::Value {
                let min = *self.start() as f32;
                let max = *self.end() as f32;
                let value = min + pct * (max - min);
                step.map_or(value, |s| (value / s as f32).round() * s as f32)
                    .clamp(min, max) as Self::Value
            }

            fn set_value(&self, value: Self::Value) -> f32 {
                let min = *self.start() as f32;
                let max = *self.end() as f32;
                if max <= min {
                    0.0
                } else {
                    (value as f32 - min) / (max - min)
                }
            }
        }
    };
}

impl_track_value_range!(u8);
impl_track_value_range!(u16);
impl_track_value_range!(u32);
impl_track_value_range!(u64);
impl_track_value_range!(u128);
impl_track_value_range!(usize);
impl_track_value_range!(i8);
impl_track_value_range!(i16);
impl_track_value_range!(i32);
impl_track_value_range!(i64);
impl_track_value_range!(i128);
impl_track_value_range!(isize);
impl_track_value_range_inclusive_int!(u8);
impl_track_value_range_inclusive_int!(u16);
impl_track_value_range_inclusive_int!(u32);
impl_track_value_range_inclusive_int!(u64);
impl_track_value_range_inclusive_int!(u128);
impl_track_value_range_inclusive_int!(usize);
impl_track_value_range_inclusive_int!(i8);
impl_track_value_range_inclusive_int!(i16);
impl_track_value_range_inclusive_int!(i32);
impl_track_value_range_inclusive_int!(i64);
impl_track_value_range_inclusive_int!(i128);
impl_track_value_range_inclusive_int!(isize);
impl_track_value_range_inclusive_float!(f32);
impl_track_value_range_inclusive_float!(f64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_endpoints() {
        let r = 0..10; // valid indices 0..=9
        assert_eq!(r.get_value(None, 0.0), 0);
        assert_eq!(r.get_value(None, 1.0), 9);
    }

    #[test]
    fn range_round_trip() {
        let r = 0..10;
        for v in r.clone() {
            let pct = r.set_value(v);
            assert_eq!(r.get_value(None, pct), v);
        }
    }

    #[test]
    fn range_single_valid_index_no_panic() {
        let r = 5..6; // only index 5 is valid
        let pct = r.set_value(5);
        assert!(pct.is_finite());
        assert_eq!(r.get_value(None, pct), 5);
    }

    #[test]
    fn range_step_snaps_to_grid() {
        let r = 0..21; // 0..=20
        let v = r.get_value(Some(5), 0.5); // raw ~10, step 5 -> should land on 10
        assert_eq!(v % 5, 0);
    }

    // --- RangeInclusive, integer ---

    #[test]
    fn range_inclusive_int_endpoints() {
        let r = 10..=50;
        assert_eq!(r.get_value(None, 0.0), 10);
        assert_eq!(r.get_value(None, 1.0), 50);
    }

    #[test]
    fn range_inclusive_int_round_trip() {
        let r = 10..=50;
        for v in [10, 20, 35, 50] {
            let pct = r.set_value(v);
            assert_eq!(r.get_value(None, pct), v);
        }
    }

    #[test]
    fn range_inclusive_int_degenerate_no_panic() {
        let r = 5..=5;
        let pct = r.set_value(5);
        assert!(pct.is_finite());
        assert_eq!(r.get_value(None, 0.5), 5);
    }

    #[test]
    fn range_inclusive_int_step() {
        let r = 0..=100;
        assert_eq!(r.get_value(Some(25), 0.27), 25); // 27 rounds down to nearest 25
        assert_eq!(r.get_value(Some(25), 0.4), 50); // 40 rounds up to nearest 25
    }

    // --- RangeInclusive, float ---

    #[test]
    fn range_inclusive_float_endpoints() {
        let r = 0.0..=1.0;
        assert_eq!(r.get_value(None, 0.0), 0.0);
        assert_eq!(r.get_value(None, 1.0), 1.0);
    }

    #[test]
    fn range_inclusive_float_preserves_precision() {
        let r = 0.0f32..=10.0;
        let v = r.get_value(None, 0.5);
        assert!((v - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn range_inclusive_float_step() {
        let r = 10.0f32..=50.0;
        let v = r.get_value(Some(0.5), 0.27); // raw 20.8 -> nearest 0.5 -> 21.0
        assert!((v - 21.0).abs() < 1e-6);
    }

    // --- pathological step ---

    #[test]
    fn zero_step_does_not_produce_nan() {
        let r = 0..=10;
        let v = r.get_value(Some(0), 0.5);
        assert!(
            v >= 0 && v <= 10,
            "expected fallback/clamped value, got {v}"
        );
    }
}
