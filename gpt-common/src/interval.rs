#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Boundary {
    Open,
    Closed,
}

#[derive(Debug, PartialEq, Clone)]
struct OneInterval {
    lo_boundary: Boundary,
    lo: f32,
    hi: f32,
    hi_boundary: Boundary,
}

impl OneInterval {
    fn contains_point(&self, point: f32) -> bool {
        (self.lo < point && point < self.hi)
            || (self.lo == point && self.lo_boundary == Boundary::Closed)
            || (self.hi == point && self.hi_boundary == Boundary::Closed)
    }

    fn intersects_with(&self, other: &OneInterval) -> bool {
        let doesnt_intersect = (self.lo > other.hi || other.lo > self.hi)
            || self.lo == other.hi
                && (self.lo_boundary == Boundary::Open || other.hi_boundary == Boundary::Open)
            || other.lo == self.hi
                && (other.lo_boundary == Boundary::Open || self.hi_boundary == Boundary::Open);

        !doesnt_intersect
    }

    fn intersect(&self, other: &OneInterval) -> Option<OneInterval> {
        if !self.intersects_with(other) {
            None
        } else {
            let bigger_lo = if self.lo > other.lo { self } else { other };
            let smaller_hi = if self.hi < other.hi { self } else { other };

            Some(OneInterval {
                lo_boundary: bigger_lo.lo_boundary,
                lo: bigger_lo.lo,
                hi: smaller_hi.hi,
                hi_boundary: smaller_hi.hi_boundary,
            })
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Interval {
    intervals: Vec<OneInterval>,
}

// TODO: There could be a more pragmatic rust solution
#[derive(Debug)]
pub enum IntervalError {
    LoIsGreaterThanHi,
}

impl Interval {
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
                intervals: vec![OneInterval {
                    lo_boundary,
                    lo,
                    hi,
                    hi_boundary,
                }],
            })
        }
    }

    pub fn intersects_with(&self, other: &Interval) -> bool {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::OneInterval;
    use crate::parser::interval;

    fn int(input: &str) -> OneInterval {
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
                "OneInterval.contains_point failed: {:?}.contains_point({:?}) should be {:?}",
                interval,
                point,
                expected
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
        ];

        for (this, that, expected) in test_cases {
            assert_eq!(
                this.intersects_with(&that),
                expected,
                "OneInterval.intersects_with failed: {:?}.intersects_with({:?}) should be {:?}",
                this,
                that,
                expected
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
        ];

        for (this, that, expected) in test_cases {
            assert_eq!(
                this.intersect(&that),
                expected,
                "OneInterval.intersect failed: {:?}.intersect({:?}) should be {:?}",
                this,
                that,
                expected
            );
        }
    }
}
