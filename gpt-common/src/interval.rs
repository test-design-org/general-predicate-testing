use std::fmt;

pub trait Intersectable {
    fn intersects_with(&self, other: &Self) -> bool;

    fn intersect(&self, other: &Self) -> Option<Self>
    where
        Self: Sized;
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Boundary {
    Open,
    Closed,
}

/// Represents one interval with boundaries, a low value and a high value
#[derive(PartialEq, Clone)]
pub struct Interval {
    pub lo_boundary: Boundary,
    pub lo: f32,
    pub hi: f32,
    pub hi_boundary: Boundary,
}

impl Interval {
    fn contains_point(&self, point: f32) -> bool {
        (self.lo < point && point < self.hi)
            || (self.lo == point && self.lo_boundary == Boundary::Closed)
            || (self.hi == point && self.hi_boundary == Boundary::Closed)
    }

    pub fn new(
        lo_boundary: Boundary,
        lo: f32,
        hi: f32,
        hi_boundary: Boundary,
    ) -> Result<Interval, IntervalError> {
        if lo > hi {
            Err(IntervalError::LoIsGreaterThanHi)
        } else {
            Ok(Interval {
                lo_boundary,
                lo,
                hi,
                hi_boundary,
            })
        }
    }

    pub fn new_closed(lo: f32, hi: f32) -> Result<Interval, IntervalError> {
        Interval::new(Boundary::Closed, lo, hi, Boundary::Closed)
    }

    pub fn new_closed_point(point: f32) -> Interval {
        Interval {
            lo_boundary: Boundary::Closed,
            lo: point,
            hi: point,
            hi_boundary: Boundary::Closed,
        }
    }
}

impl Intersectable for Interval {
    fn intersects_with(&self, other: &Interval) -> bool {
        let doesnt_intersect = (self.lo > other.hi || other.lo > self.hi)
            || self.lo == other.hi
                && (self.lo_boundary == Boundary::Open || other.hi_boundary == Boundary::Open)
            || other.lo == self.hi
                && (other.lo_boundary == Boundary::Open || self.hi_boundary == Boundary::Open);

        !doesnt_intersect
    }

    fn intersect(&self, other: &Interval) -> Option<Interval> {
        if !self.intersects_with(other) {
            None
        } else {
            let bigger_lo = if self.lo > other.lo { self } else { other };
            let smaller_hi = if self.hi < other.hi { self } else { other };

            Some(Interval {
                lo_boundary: bigger_lo.lo_boundary,
                lo: bigger_lo.lo,
                hi: smaller_hi.hi,
                hi_boundary: smaller_hi.hi_boundary,
            })
        }
    }
}

impl fmt::Debug for Interval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lo_boundary = match self.lo_boundary {
            Boundary::Open => "(",
            Boundary::Closed => "[",
        };
        let hi_boundary = match self.hi_boundary {
            Boundary::Open => ")",
            Boundary::Closed => "]",
        };

        write!(f, "{}{}, {}{}", lo_boundary, self.lo, self.hi, hi_boundary)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct MultiInterval {
    /// `intervals` is always sorted in ascending order and there are no overlapping intervals
    intervals: Vec<Interval>,
}

// TODO: There could be a more pragmatic rust solution
#[derive(Debug)]
pub enum IntervalError {
    LoIsGreaterThanHi,
}

impl MultiInterval {
    pub fn new(
        lo_boundary: Boundary,
        lo: f32,
        hi: f32,
        hi_boundary: Boundary,
    ) -> Result<MultiInterval, IntervalError> {
        Ok(MultiInterval {
            intervals: vec![Interval::new(lo_boundary, lo, hi, hi_boundary)?],
        })
    }

    pub fn new_closed(lo: f32, hi: f32) -> Result<MultiInterval, IntervalError> {
        MultiInterval::new(Boundary::Closed, lo, hi, Boundary::Closed)
    }

    pub fn highest_hi(&self) -> f32 {
        self.intervals
            .last()
            .expect("Interval should always contain an interval")
            .hi
    }

    pub fn lowest_lo(&self) -> f32 {
        self.intervals
            .first()
            .expect("Interval should always contain an interval")
            .lo
    }

    pub fn highest_boundary(&self) -> Boundary {
        self.intervals
            .last()
            .expect("Interval should always contain an interval")
            .hi_boundary
    }

    pub fn lowest_boundary(&self) -> Boundary {
        self.intervals
            .first()
            .expect("Interval should always contain an interval")
            .lo_boundary
    }
}

impl Intersectable for MultiInterval {
    // TODO: This could be sped up, because the interval Vecs are sorted
    // It could be a step-by-step comparison
    fn intersects_with(&self, other: &MultiInterval) -> bool {
        for x in self.intervals.iter() {
            for y in other.intervals.iter() {
                if x.intersects_with(y) {
                    return true;
                }
            }
        }

        false
    }

    fn intersect(&self, other: &MultiInterval) -> Option<MultiInterval> {
        let mut intersected_intervals: Vec<Interval> = self
            .intervals
            .iter()
            .flat_map(|x| other.intervals.iter().map(|y| x.intersect(y)))
            .filter_map(|x| x)
            .collect();

        intersected_intervals.sort_unstable_by(|a, b| {
            a.lo.partial_cmp(&b.lo)
                .expect("f32::NaN should not be the lo value of intervals")
        });

        if intersected_intervals.is_empty() {
            None
        } else {
            Some(MultiInterval {
                intervals: intersected_intervals,
            })
        }
    }
}

#[cfg(test)]
mod test {
    use super::Interval;
    use crate::{
        interval::{Intersectable, MultiInterval},
        parser::interval,
    };

    fn int(input: &str) -> Interval {
        let (_, x) = interval(input).unwrap();
        x.intervals.first().unwrap().clone()
    }

    #[test]
    fn test_contains_point() {
        let test_cases = vec![
            (int("[5, 10]"), 4.0, false),
            (int("(5, 10]"), 5.0, false),
            (int("[5, 10]"), 5.0, true),
            (int("[5, 10]"), 7.0, true),
            (int("[5, 10]"), 10.0, true),
            (int("[5, 10)"), 10.0, false),
            (int("[5, 10)"), 11.0, false),
        ];

        for (interval, point, expected) in test_cases {
            assert_eq!(
                interval.contains_point(point),
                expected,
                "OneInterval.contains_point failed: {interval:?}.contains_point({point:?}) should be {expected:?}",
            );
        }
    }

    #[test]
    fn test_OneInterval_intersects_with() {
        let test_cases = vec![
            // self.hi equals other.lo
            (int("[0, 10]"), int("[10, 20]"), true),
            (int("[0, 10]"), int("(10, 20]"), false),
            (int("[0, 10)"), int("[10, 20]"), false),
            (int("[0, 10)"), int("(10, 20]"), false),
            // self.lo equals other.hi
            (int("[10, 20]"), int("[0, 10]"), true),
            (int("(10, 20]"), int("[0, 10]"), false),
            (int("[10, 20]"), int("[0, 10)"), false),
            (int("(10, 20]"), int("[0, 10)"), false),
            // self.hi inside other == other.lo inside self
            (int("[0, 10]"), int("[5, 20]"), true),
            (int("[0, 10]"), int("(5, 20]"), true),
            (int("[0, 10)"), int("[5, 20]"), true),
            (int("[0, 10)"), int("(5, 20]"), true),
            // self.lo inside other == other.hi inside self
            (int("[5, 20]"), int("[0, 10]"), true),
            (int("(5, 20]"), int("[0, 10]"), true),
            (int("[5, 20]"), int("[0, 10)"), true),
            (int("(5, 20]"), int("[0, 10)"), true),
            // self inside other
            (int("[10, 20]"), int("[0, 30]"), true),
            (int("[10, 20)"), int("[0, 30]"), true),
            (int("(10, 20]"), int("[0, 30]"), true),
            (int("(10, 20)"), int("[0, 30]"), true),
            // other inside self
            (int("[0, 30]"), int("[10, 20]"), true),
            (int("[0, 30]"), int("[10, 20)"), true),
            (int("[0, 30]"), int("(10, 20]"), true),
            (int("[0, 30]"), int("(10, 20)"), true),
            // self.lo > other.hi
            (int("[20, 30]"), int("[0, 10]"), false),
            (int("[20, 30]"), int("[0, 10)"), false),
            (int("(20, 30]"), int("[0, 10]"), false),
            (int("(20, 30]"), int("[0, 10)"), false),
            // other.lo > self.hi
            (int("[0, 10]"), int("[20, 30]"), false),
            (int("[0, 10)"), int("[20, 30]"), false),
            (int("[0, 10]"), int("(20, 30]"), false),
            (int("[0, 10)"), int("(20, 30]"), false),
            // TODO: Inf, -Inf
        ];

        for (this, that, expected) in test_cases {
            assert_eq!(
                this.intersects_with(&that),
                expected,
                "OneInterval.intersects_with failed: {this:?}.intersects_with({that:?}) should be {expected:?}",
            );
        }
    }

    #[test]
    fn test_OneInterval_intersect() {
        let test_cases = vec![
            // self.hi equals other.lo
            (int("[0, 10]"), int("[10, 20]"), Some(int("[10, 10]"))),
            (int("[0, 10]"), int("(10, 20]"), None),
            (int("[0, 10)"), int("[10, 20]"), None),
            (int("[0, 10)"), int("(10, 20]"), None),
            // self.lo equals other.hi
            (int("[10, 20]"), int("[0, 10]"), Some(int("[10, 10]"))),
            (int("(10, 20]"), int("[0, 10]"), None),
            (int("[10, 20]"), int("[0, 10)"), None),
            (int("(10, 20]"), int("[0, 10)"), None),
            // self.hi inside other == other.lo inside self
            (int("[0, 10]"), int("[5, 20]"), Some(int("[5, 10]"))),
            (int("[0, 10]"), int("(5, 20]"), Some(int("(5, 10]"))),
            (int("[0, 10)"), int("[5, 20]"), Some(int("[5, 10)"))),
            (int("[0, 10)"), int("(5, 20]"), Some(int("(5, 10)"))),
            // self.lo inside other == other.hi inside self
            (int("[5, 20]"), int("[0, 10]"), Some(int("[5, 10]"))),
            (int("(5, 20]"), int("[0, 10]"), Some(int("(5, 10]"))),
            (int("[5, 20]"), int("[0, 10)"), Some(int("[5, 10)"))),
            (int("(5, 20]"), int("[0, 10)"), Some(int("(5, 10)"))),
            // self inside other
            (int("[10, 20]"), int("[0, 30]"), Some(int("[10, 20]"))),
            (int("[10, 20)"), int("[0, 30]"), Some(int("[10, 20)"))),
            (int("(10, 20]"), int("[0, 30]"), Some(int("(10, 20]"))),
            (int("(10, 20)"), int("[0, 30]"), Some(int("(10, 20)"))),
            // other inside self
            (int("[0, 30]"), int("[10, 20]"), Some(int("[10, 20]"))),
            (int("[0, 30]"), int("[10, 20)"), Some(int("[10, 20)"))),
            (int("[0, 30]"), int("(10, 20]"), Some(int("(10, 20]"))),
            (int("[0, 30]"), int("(10, 20)"), Some(int("(10, 20)"))),
            // self.lo > other.hi
            (int("[20, 30]"), int("[0, 10]"), None),
            (int("[20, 30]"), int("[0, 10)"), None),
            (int("(20, 30]"), int("[0, 10]"), None),
            (int("(20, 30]"), int("[0, 10)"), None),
            // other.lo > self.hi
            (int("[0, 10]"), int("[20, 30]"), None),
            (int("[0, 10)"), int("[20, 30]"), None),
            (int("[0, 10]"), int("(20, 30]"), None),
            (int("[0, 10)"), int("(20, 30]"), None),
            // TODO: Inf, -Inf
        ];

        for (this, that, expected) in test_cases {
            assert_eq!(
                this.intersect(&that),
                expected,
                "OneInterval.intersect failed: {this:?}.intersect({that:?}) should be {expected:?}",
            );
        }
    }

    #[test]
    fn test_Interval_intersect() {
        let test_cases = vec![
            // zero elements
            (vec![], vec![], None),
            // one elem - zero elem
            (vec![int("[0, 10]")], vec![], None),
            (vec![], vec![int("[0, 10]")], None),
            // one element, has intersection
            (
                vec![int("[0, 10]")],
                vec![int("[5, 20]")],
                Some(vec![int("[5, 10]")]),
            ),
            // one element - two elements, has intersection
            (
                vec![int("[0, 10]")],
                vec![int("[5, 20]"), int("[100, 200]")],
                Some(vec![int("[5, 10]")]),
            ),
            (
                vec![int("[5, 20]"), int("[100, 200]")],
                vec![int("[0, 10]")],
                Some(vec![int("[5, 10]")]),
            ),
            // contains multiple intervals
            (
                vec![int("[0, 100]")],
                vec![int("[10, 20]"), int("[30, 40]")],
                Some(vec![int("[10, 20]"), int("[30, 40]")]),
            ),
            // overlaps with multiple intervals
            (
                vec![int("[20, 50]")],
                vec![int("[0, 30]"), int("[40, 60]")],
                Some(vec![int("[20, 30]"), int("[40, 50]")]),
            ),
            // multiple elements
            (
                vec![int("(-Inf, 10]"), int("[20, 30]"), int("[40, 50]")],
                vec![int("(-Inf, 10)"), int("[15, 25]"), int("(26, 35]")],
                Some(vec![int("(-Inf, 10)"), int("[20, 25]"), int("(26, 30]")]),
            ),
        ];

        for (a, b, expected_vec) in test_cases {
            let this = MultiInterval { intervals: a };
            let that = MultiInterval { intervals: b };
            let expected = expected_vec.map(|intervals| MultiInterval { intervals });

            assert_eq!(
                this.intersect(&that),
                expected,
                "Interval.intersect failed: {this:?}.intersect({that:?}) should be {expected:?}",
            );
        }
    }
}
