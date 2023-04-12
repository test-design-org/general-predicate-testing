use std::{cmp::Ordering, fmt};

pub trait Intersectable {
    fn intersects_with(&self, other: &Self) -> bool;

    fn intersect(&self, other: &Self) -> Option<Self>
    where
        Self: Sized;
}

pub trait Unionable<TOther, TResult> {
    fn union(&self, other: &TOther) -> TResult;
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Boundary {
    Open,
    Closed,
}

impl Boundary {
    pub const fn inverse(&self) -> Self {
        match self {
            Self::Open => Self::Closed,
            Self::Closed => Self::Open,
        }
    }
}

/// Represents one interval with boundaries, a low value and a high value
#[derive(PartialEq, Clone, Copy)]
pub struct Interval {
    pub lo_boundary: Boundary,
    pub lo: f32,
    pub hi: f32,
    pub hi_boundary: Boundary,
}

impl Interval {
    pub fn contains_point(&self, point: f32) -> bool {
        !self.is_empty()
            && ((self.lo < point && point < self.hi)
                || (self.lo == point && self.lo_boundary == Boundary::Closed)
                || (self.hi == point && self.hi_boundary == Boundary::Closed))
    }

    pub fn contains(&self, _other: &Self) -> bool {
        todo!()
    }

    /// Creates an interval. If lo or hi would be infinity, that side will be open, no matter what boundary was passed to it,
    /// because that is the semantically correct way to handle it.
    pub fn new(
        lo_boundary: Boundary,
        lo: f32,
        hi: f32,
        hi_boundary: Boundary,
    ) -> Result<Self, IntervalError> {
        if lo > hi {
            Err(IntervalError::LoIsGreaterThanHi)
        } else {
            Ok(Self {
                lo_boundary: if lo == f32::NEG_INFINITY {
                    Boundary::Open
                } else {
                    lo_boundary
                },
                lo,
                hi,
                hi_boundary: if hi == f32::INFINITY {
                    Boundary::Open
                } else {
                    hi_boundary
                },
            })
        }
    }

    pub fn new_closed(lo: f32, hi: f32) -> Result<Self, IntervalError> {
        Self::new(Boundary::Closed, lo, hi, Boundary::Closed)
    }

    #[must_use]
    pub const fn new_closed_point(point: f32) -> Self {
        Self {
            lo_boundary: Boundary::Closed,
            lo: point,
            hi: point,
            hi_boundary: Boundary::Closed,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.lo == self.hi
            && (self.lo_boundary == Boundary::Open || self.hi_boundary == Boundary::Open)
    }

    fn lo_cmp(&self, other: &Self) -> Ordering {
        match self.lo.partial_cmp(&other.lo).unwrap() {
            std::cmp::Ordering::Equal => match (self.lo_boundary, other.lo_boundary) {
                (Boundary::Open, Boundary::Closed) => std::cmp::Ordering::Greater,
                (Boundary::Closed, Boundary::Open) => std::cmp::Ordering::Less,
                (Boundary::Open, Boundary::Open) => std::cmp::Ordering::Equal,
                (Boundary::Closed, Boundary::Closed) => std::cmp::Ordering::Equal,
            },
            x => x,
        }
    }

    fn hi_cmp(&self, other: &Self) -> Ordering {
        match self.hi.partial_cmp(&other.hi).unwrap() {
            std::cmp::Ordering::Equal => match (self.hi_boundary, other.hi_boundary) {
                (Boundary::Open, Boundary::Closed) => std::cmp::Ordering::Less,
                (Boundary::Closed, Boundary::Open) => std::cmp::Ordering::Greater,
                (Boundary::Open, Boundary::Open) => std::cmp::Ordering::Equal,
                (Boundary::Closed, Boundary::Closed) => std::cmp::Ordering::Equal,
            },
            x => x,
        }
    }
}

impl Intersectable for Interval {
    fn intersects_with(&self, other: &Self) -> bool {
        let doesnt_intersect = (self.lo > other.hi || other.lo > self.hi)
            || self.lo == other.hi
                && (self.lo_boundary == Boundary::Open || other.hi_boundary == Boundary::Open)
            || other.lo == self.hi
                && (other.lo_boundary == Boundary::Open || self.hi_boundary == Boundary::Open);

        !doesnt_intersect
    }

    fn intersect(&self, other: &Self) -> Option<Self> {
        if !self.intersects_with(other) {
            return None;
        }

        // let bigger_lo = if self.lo > other.lo { self } else { other };
        // let smaller_hi = if self.hi < other.hi { self } else { other };
        let bigger_lo = if self.lo_cmp(other) == Ordering::Greater {
            self
        } else {
            other
        };
        let smaller_hi = if self.hi_cmp(other) == Ordering::Less {
            self
        } else {
            other
        };

        Some(Self {
            lo_boundary: bigger_lo.lo_boundary,
            lo: bigger_lo.lo,
            hi: smaller_hi.hi,
            hi_boundary: smaller_hi.hi_boundary,
        })
    }
}

impl fmt::Display for Interval {
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

impl fmt::Debug for Interval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(PartialEq, Clone)]
pub struct MultiInterval {
    /// `intervals` is always sorted in ascending order and there are no overlapping intervals
    pub(crate) intervals: Vec<Interval>,
}

// TODO: There could be a more pragmatic rust solution
#[derive(Debug)]
pub enum IntervalError {
    LoIsGreaterThanHi,
}

// TODO: implement a simplifier function, which
//          - removes empty intervals, like (0,0)
//          - merges bordering intervals, like [10, 20] [20, 30] becomes [10, 30]
impl MultiInterval {
    pub fn new(
        lo_boundary: Boundary,
        lo: f32,
        hi: f32,
        hi_boundary: Boundary,
    ) -> Result<Self, IntervalError> {
        Ok(Self {
            intervals: vec![Interval::new(lo_boundary, lo, hi, hi_boundary)?],
        })
    }

    pub fn from_interval(interval: Interval) -> Self {
        Self {
            intervals: vec![interval],
        }
    }

    pub fn from_intervals(intervals: Vec<Interval>) -> Self {
        let mut multi_interval = Self { intervals };

        multi_interval.clean();

        multi_interval
    }

    pub const fn new_empty() -> Self {
        Self {
            intervals: Vec::new(),
        }
    }

    pub fn new_closed(lo: f32, hi: f32) -> Result<Self, IntervalError> {
        Self::new(Boundary::Closed, lo, hi, Boundary::Closed)
    }

    pub fn new_closed_point(num: f32) -> Self {
        Self::new(Boundary::Closed, num, num, Boundary::Closed)
            .expect("Closed point creation should not cause any errors")
    }

    #[must_use]
    fn highest_hi(&self) -> f32 {
        self.intervals
            .last()
            .expect("Interval should always contain an interval")
            .hi
    }

    #[must_use]
    fn lowest_lo(&self) -> f32 {
        self.intervals
            .first()
            .expect("Interval should always contain an interval")
            .lo
    }

    #[must_use]
    fn highest_boundary(&self) -> Boundary {
        self.intervals
            .last()
            .expect("Interval should always contain an interval")
            .hi_boundary
    }

    #[must_use]
    fn lowest_boundary(&self) -> Boundary {
        self.intervals
            .first()
            .expect("Interval should always contain an interval")
            .lo_boundary
    }

    pub fn is_empty(&self) -> bool {
        self.intervals.is_empty()
    }

    pub fn inverse(&self) -> Self {
        if self.intervals.is_empty() {
            return Self {
                intervals: vec![Interval {
                    lo_boundary: Boundary::Open,
                    lo: f32::NEG_INFINITY,
                    hi: f32::INFINITY,
                    hi_boundary: Boundary::Open,
                }],
            };
        }

        let mut new_intervals = Vec::new();

        if self.lowest_lo() != f32::NEG_INFINITY {
            new_intervals.push(Interval {
                lo_boundary: Boundary::Open,
                lo: f32::NEG_INFINITY,
                hi: self.lowest_lo(),
                hi_boundary: self.lowest_boundary().inverse(),
            })
        }

        new_intervals.append(
            &mut self
                .intervals
                .windows(2)
                .map(|x| {
                    let (a, b) = (x[0], x[1]);

                    Interval {
                        lo_boundary: a.hi_boundary.inverse(),
                        lo: a.hi,
                        hi: b.lo,
                        hi_boundary: b.lo_boundary.inverse(),
                    }
                })
                .collect(),
        );

        if self.highest_hi() != f32::INFINITY {
            new_intervals.push(Interval {
                lo_boundary: self.highest_boundary().inverse(),
                lo: self.highest_hi(),
                hi: f32::INFINITY,
                hi_boundary: Boundary::Open,
            })
        }

        Self {
            intervals: new_intervals,
        }
    }

    fn clean(&mut self) {
        // Removing empty intervals
        self.intervals.retain(|x| !x.is_empty());

        // Sort the intervals
        self.intervals.sort_by(|a, b| a.lo_cmp(b));

        // Merging overlapping intervals
        if self.intervals.len() >= 2 {
            // Start at the back and go backwards, because then we can remove indicies easily from the end
            for i in (0..=(self.intervals.len() - 2)).rev() {
                let (left, right) = (self.intervals[i], self.intervals[i + 1]);

                // left.lo <= right.lo beacuse of the sort
                if left.intersects_with(&right) {
                    if left.hi_cmp(&right) == Ordering::Greater {
                        self.intervals[i] = Interval {
                            lo_boundary: left.lo_boundary,
                            lo: left.lo,
                            hi: left.hi,
                            hi_boundary: left.hi_boundary,
                        };
                    } else {
                        self.intervals[i] = Interval {
                            lo_boundary: left.lo_boundary,
                            lo: left.lo,
                            hi: right.hi,
                            hi_boundary: right.hi_boundary,
                        };
                    }
                    self.intervals.remove(i + 1);
                }
            }
        }
    }
}

impl Intersectable for MultiInterval {
    // TODO: This could be sped up, because the interval Vecs are sorted
    // It could be a step-by-step comparison
    fn intersects_with(&self, other: &Self) -> bool {
        for x in &self.intervals {
            for y in &other.intervals {
                if x.intersects_with(y) {
                    return true;
                }
            }
        }

        false
    }

    fn intersect(&self, other: &Self) -> Option<Self> {
        let mut intersected_intervals: Vec<Interval> = self
            .intervals
            .iter()
            .flat_map(|x| other.intervals.iter().map(|y| x.intersect(y)))
            .flatten()
            .collect();

        intersected_intervals.sort_unstable_by(|a, b| {
            a.lo.partial_cmp(&b.lo)
                .expect("f32::NaN should not be the lo value of intervals")
        });

        if intersected_intervals.is_empty() {
            None
        } else {
            Some(Self {
                intervals: intersected_intervals,
            })
        }
    }
}

impl fmt::Display for MultiInterval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            return Ok(());
        }
        write!(f, "{}", self.intervals[0])?;

        for interval in self.intervals.iter().skip(1) {
            write!(f, " {}", interval)?;
        }

        Ok(())
    }
}

impl fmt::Debug for MultiInterval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl Unionable<Self, Self> for MultiInterval {
    fn union(&self, other: &Self) -> Self {
        let mut intervals = self.intervals.clone();
        intervals.append(&mut other.intervals.clone());

        let mut multi_interval = Self { intervals };
        multi_interval.clean();

        multi_interval
    }
}

impl Unionable<Self, MultiInterval> for Interval {
    fn union(&self, other: &Self) -> MultiInterval {
        MultiInterval::from_intervals(vec![*self, *other])
    }
}

#[cfg(test)]
pub(crate) mod test {
    use std::{cmp::Ordering, str::FromStr};

    use nom::{combinator::complete, multi::many0};
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::{Intersectable, Interval, MultiInterval};
    use crate::parser::interval;
    use Ordering::{Equal, Greater, Less};

    pub fn int(input: &str) -> Interval {
        let (_, x) = interval(input).unwrap();
        *x.intervals.first().unwrap()
    }

    pub fn multiint(input: &str) -> MultiInterval {
        let (_, x) = many0(complete(interval))(input.trim()).unwrap();
        let intervals = x
            .into_iter()
            .map(|y| *y.intervals.first().unwrap())
            .collect();

        MultiInterval::from_intervals(intervals)
    }

    impl FromStr for Interval {
        type Err = ();

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(int(s))
        }
    }

    impl FromStr for MultiInterval {
        type Err = ();

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(multiint(s))
        }
    }

    #[rstest]
    #[case("[5, 10]", 4.0, false)]
    #[case("(5, 10]", 5.0, false)]
    #[case("[5, 10]", 5.0, true)]
    #[case("[5, 10]", 7.0, true)]
    #[case("[5, 10]", 10.0, true)]
    #[case("[5, 10)", 10.0, false)]
    #[case("[5, 10)", 11.0, false)]
    fn test_contains_point(#[case] interval: Interval, #[case] point: f32, #[case] expected: bool) {
        assert_eq!(
            interval.contains_point(point),
            expected,
            "Interval.contains_point failed: {interval}.contains_point({point}) should be {expected}",
        );
    }

    #[rstest]
    // self.hi equals other.lo
    #[case("[0, 10]", "[10, 20]", true)]
    #[case("[0, 10]", "(10, 20]", false)]
    #[case("[0, 10)", "[10, 20]", false)]
    #[case("[0, 10)", "(10, 20]", false)]
    // self.lo equals other.hi
    #[case("[10, 20]", "[0, 10]", true)]
    #[case("(10, 20]", "[0, 10]", false)]
    #[case("[10, 20]", "[0, 10)", false)]
    #[case("(10, 20]", "[0, 10)", false)]
    // self.hi inside other == other.lo inside self
    #[case("[0, 10]", "[5, 20]", true)]
    #[case("[0, 10]", "(5, 20]", true)]
    #[case("[0, 10)", "[5, 20]", true)]
    #[case("[0, 10)", "(5, 20]", true)]
    // self.lo inside other == other.hi inside self
    #[case("[5, 20]", "[0, 10]", true)]
    #[case("(5, 20]", "[0, 10]", true)]
    #[case("[5, 20]", "[0, 10)", true)]
    #[case("(5, 20]", "[0, 10)", true)]
    // self inside other
    #[case("[10, 20]", "[0, 30]", true)]
    #[case("[10, 20)", "[0, 30]", true)]
    #[case("(10, 20]", "[0, 30]", true)]
    #[case("(10, 20)", "[0, 30]", true)]
    // other inside self
    #[case("[0, 30]", "[10, 20]", true)]
    #[case("[0, 30]", "[10, 20)", true)]
    #[case("[0, 30]", "(10, 20]", true)]
    #[case("[0, 30]", "(10, 20)", true)]
    // self.lo > other.hi
    #[case("[20, 30]", "[0, 10]", false)]
    #[case("[20, 30]", "[0, 10)", false)]
    #[case("(20, 30]", "[0, 10]", false)]
    #[case("(20, 30]", "[0, 10)", false)]
    // other.lo > self.hi
    #[case("[0, 10]", "[20, 30]", false)]
    #[case("[0, 10)", "[20, 30]", false)]
    #[case("[0, 10]", "(20, 30]", false)]
    #[case("[0, 10)", "(20, 30]", false)]
    // self.lo == other.lo
    #[case("[0, 10]", "[0, 20]", true)]
    #[case("[0, 10]", "(0, 20]", true)]
    #[case("(0, 10]", "[0, 20]", true)]
    #[case("(0, 10]", "(0, 20]", true)]
    // self.hi == other.hi
    #[case("[10, 20]", "[0, 20]", true)]
    #[case("[10, 20)", "[0, 20]", true)]
    #[case("[10, 20]", "[0, 20)", true)]
    #[case("[10, 20)", "[0, 20)", true)]
    // TODO: Inf, -Inf
    // TODO: What about empty intervals like (0,0) (0,0)?
    fn test_interval_intersects_with(
        #[case] this: Interval,
        #[case] that: Interval,
        #[case] expected: bool,
    ) {
        assert_eq!(
            this.intersects_with(&that),
            expected,
            "Interval.intersects_with failed: {this}.intersects_with({that}) should be {expected}",
        );
    }

    #[rstest]
    // self.hi equals other.lo
    #[case("[0, 10]", "[10, 20]", Some("[10, 10]"))]
    #[case("[0, 10]", "(10, 20]", None)]
    #[case("[0, 10)", "[10, 20]", None)]
    #[case("[0, 10)", "(10, 20]", None)]
    // self.lo equals other.hi
    #[case("[10, 20]", "[0, 10]", Some("[10, 10]"))]
    #[case("(10, 20]", "[0, 10]", None)]
    #[case("[10, 20]", "[0, 10)", None)]
    #[case("(10, 20]", "[0, 10)", None)]
    // self.hi inside other == other.lo inside self
    #[case("[0, 10]", "[5, 20]", Some("[5, 10]"))]
    #[case("[0, 10]", "(5, 20]", Some("(5, 10]"))]
    #[case("[0, 10)", "[5, 20]", Some("[5, 10)"))]
    #[case("[0, 10)", "(5, 20]", Some("(5, 10)"))]
    // self.lo inside other == other.hi inside self
    #[case("[5, 20]", "[0, 10]", Some("[5, 10]"))]
    #[case("(5, 20]", "[0, 10]", Some("(5, 10]"))]
    #[case("[5, 20]", "[0, 10)", Some("[5, 10)"))]
    #[case("(5, 20]", "[0, 10)", Some("(5, 10)"))]
    // self inside other
    #[case("[10, 20]", "[0, 30]", Some("[10, 20]"))]
    #[case("[10, 20)", "[0, 30]", Some("[10, 20)"))]
    #[case("(10, 20]", "[0, 30]", Some("(10, 20]"))]
    #[case("(10, 20)", "[0, 30]", Some("(10, 20)"))]
    // other inside self
    #[case("[0, 30]", "[10, 20]", Some("[10, 20]"))]
    #[case("[0, 30]", "[10, 20)", Some("[10, 20)"))]
    #[case("[0, 30]", "(10, 20]", Some("(10, 20]"))]
    #[case("[0, 30]", "(10, 20)", Some("(10, 20)"))]
    // self.lo > other.hi
    #[case("[20, 30]", "[0, 10]", None)]
    #[case("[20, 30]", "[0, 10)", None)]
    #[case("(20, 30]", "[0, 10]", None)]
    #[case("(20, 30]", "[0, 10)", None)]
    // other.lo > self.hi
    #[case("[0, 10]", "[20, 30]", None)]
    #[case("[0, 10)", "[20, 30]", None)]
    #[case("[0, 10]", "(20, 30]", None)]
    #[case("[0, 10)", "(20, 30]", None)]
    // self.lo == other.lo
    #[case("[0, 10]", "[0, 20]", Some("[0, 10]"))]
    #[case("[0, 10]", "(0, 20]", Some("(0, 10]"))]
    #[case("(0, 10]", "[0, 20]", Some("(0, 10]"))]
    #[case("(0, 10]", "(0, 20]", Some("(0, 10]"))]
    // self.hi == other.hi
    #[case("[10, 20]", "[0, 20]", Some("[10, 20]"))]
    #[case("[10, 20)", "[0, 20]", Some("[10, 20)"))]
    #[case("[10, 20]", "[0, 20)", Some("[10, 20)"))]
    #[case("[10, 20)", "[0, 20)", Some("[10, 20)"))]
    // TODO: Inf, -Inf
    // TODO: What about empty intervals like (0,0) (0,0)?
    fn test_interval_intersect(
        #[case] this: Interval,
        #[case] that: Interval,
        #[case] expected: Option<&str>,
    ) {
        assert_eq!(
            this.intersect(&that),
            expected.map(int),
            "Interval.intersect failed: {this}.intersect({that}) should be {expected:?}",
        );
    }

    #[rstest]
    #[case("(0,0)", true)]
    #[case("(0,0]", true)]
    #[case("[0,0)", true)]
    #[case("[0,0]", false)]
    #[case("(0,1)", false)]
    fn test_interval_is_empty(#[case] interval: Interval, #[case] expected: bool) {
        assert_eq!(
            interval.is_empty(),
            expected,
            "Interval.isEmpty failed: {interval}.is_empty() should be {expected}"
        );
    }

    #[rstest]
    // Same endpoint
    #[case("(0,0)", "(0,0)", Equal)]
    #[case("[0,0)", "[0,0)", Equal)]
    #[case("(0,10)", "(0,20]", Equal)]
    #[case("[0,0)", "(0,0)", Less)]
    #[case("(0,0)", "[0,0)", Greater)]
    // No matter the boundary, it is Less
    #[case("(-20,0)", "(0,0)", Less)]
    #[case("(-20,0)", "[0,0)", Less)]
    #[case("[-20,0)", "(0,0)", Less)]
    #[case("[-20,0)", "[0,0)", Less)]
    // No matter the boundary, it is Greater
    #[case("(20,30)", "(0,0)", Greater)]
    #[case("(20,30)", "[0,0)", Greater)]
    #[case("[20,30)", "(0,0)", Greater)]
    #[case("[20,30)", "[0,0)", Greater)]
    fn test_interval_lo_cmp(
        #[case] left: Interval,
        #[case] right: Interval,
        #[case] expected: Ordering,
    ) {
        assert_eq!(
            left.lo_cmp(&right),
            expected,
            "Interval.lo_cmp failed: {left}.lo_cmp({right}) should be {expected:?}"
        );
    }

    #[rstest]
    // Same endpoint
    #[case("(0,0)", "(0,0)", Equal)]
    #[case("(0,0]", "(0,0]", Equal)]
    #[case("(-10,0)", "(-20,0)", Equal)]
    #[case("(0,0)", "(0,0]", Less)]
    #[case("(0,0]", "(0,0)", Greater)]
    // No matter the boundary, it is Less
    #[case("(-30,-20)", "(0,0)", Less)]
    #[case("(-30,-20)", "(0,0]", Less)]
    #[case("(-30,-20]", "(0,0)", Less)]
    #[case("(-30,-20]", "(0,0]", Less)]
    // No matter the boundary, it is Greater
    #[case("(0,20)", "(0,0)", Greater)]
    #[case("(0,20]", "(0,0)", Greater)]
    #[case("(0,20)", "(0,0]", Greater)]
    #[case("(0,20]", "(0,0]", Greater)]
    fn test_interval_hi_cmp(
        #[case] left: Interval,
        #[case] right: Interval,
        #[case] expected: Ordering,
    ) {
        assert_eq!(
            left.hi_cmp(&right),
            expected,
            "Interval.hi_cmp failed: {left}.hi_cmp({right}) should be {expected:?}"
        );
    }

    #[rstest]
    // zero elements
    #[case("", "", None)]
    // one elem - zero elem
    #[case("[0, 10]", "", None)]
    #[case("", "[0, 10]", None)]
    // one element, has intersection
    #[case("[0, 10]", "[5, 20]", Some("[5, 10]"))]
    // one element - two elements, has intersection
    #[case("[0, 10]", "[5, 20] [100, 200]", Some("[5, 10]"))]
    #[case("[5, 20] [100, 200]", "[0, 10]", Some("[5, 10]"))]
    // contains multiple intervals
    #[case("[0, 100]", "[10, 20] [30, 40]", Some("[10, 20] [30, 40]"))]
    // overlaps with multiple intervals
    #[case("[20, 50]", "[0, 30] [40, 60]", Some("[20, 30] [40, 50]"))]
    // multiple elements
    #[case(
        "(-Inf, 10] [20, 30] [40, 50]",
        "(-Inf, 10) [15, 25] (26, 35]",
        Some("(-Inf, 10) [20, 25] (26, 30]")
    )]
    fn test_multiinterval_intersect(
        #[case] this: MultiInterval,
        #[case] that: MultiInterval,
        #[case] expected: Option<&str>,
    ) {
        assert_eq!(
            this.intersect(&that),
            expected.map(multiint),
            "MultiInterval.intersect failed: {this}.intersect({that}) should be {expected:?}",
        );
    }

    #[rstest]
    // zero elements
    #[case("", "(-Inf, Inf)")]
    #[case("(-Inf, Inf)", "")]
    // one element - -Inf on left
    #[case("(-Inf, 10)", "[10, Inf)")]
    #[case("(-Inf, 10]", "(10, Inf)")]
    // one element - no Infs on either side
    #[case("(10, 20)", "(-Inf, 10] [20, Inf)")]
    #[case("(10, 20]", "(-Inf, 10] (20, Inf)")]
    #[case("[10, 20)", "(-Inf, 10) [20, Inf)")]
    #[case("[10, 20]", "(-Inf, 10) (20, Inf)")]
    // one element - Inf on right
    #[case("(10, Inf)", "(-Inf, 10]")]
    #[case("[10, Inf)", "(-Inf, 10)")]
    // multiple elements - has Inf on either side
    #[case("(-Inf, 10) (20, Inf)", "[10, 20]")]
    #[case("(-Inf, 10) (20, 30) (40, Inf)", "[10, 20] [30, 40]")]
    // multiple elements - Inf on one side
    #[case("(-Inf, 10) [20, 30)", "[10, 20) [30, Inf)")]
    #[case(
        "(-Inf, 10)        (20, 30)        (40, 50)",
        "          [10, 20]        [30, 40]        [50, Inf)"
    )]
    #[case("(0, 10) [20, Inf)", "(-Inf, 0] [10, 20)")]
    #[case(
        "         [0, 10)        (20, 30)        (40, Inf)",
        "(-Inf, 0)       [10, 20]        [30, 40]"
    )]
    // TODO: same endpoint, like (0,0) [0,0] (0,0] [0,0)
    // complex examples
    #[case(
        "           [-42, 3)      (3, 67)         (100, 101)          [205, 607]          (700, Inf)",
        "(-Inf, -42)        [3, 3]       [67, 100]          [101, 205)          (607, 700]",
    )]
    fn test_multiinterval_inverse(
        #[case] interval: MultiInterval,
        #[case] expected: MultiInterval,
    ) {
        assert_eq!(
            interval.inverse(),
            expected,
            "MultiInterval.invert failed: {interval}.inverse() should be {expected}",
        );
    }

    #[test]
    fn test_multiinterval_axioms() {
        let input1 = multiint("[-42, 3) (3, 67) (100, 101) [205, 607] (700, Inf)");

        assert_eq!(
            input1.inverse().inverse(),
            input1,
            "The inverse of an inverse should be the original",
        );
        assert_eq!(
            input1.intersect(&multiint("(-Inf, Inf)")),
            Some(input1.clone()),
            "Intersecting something with (-Inf, Inf) should be the original"
        );
        assert!(
            !input1.intersects_with(&input1.inverse()),
            "An interval can't be intersected with its inverse"
        );
        assert_eq!(
            input1.intersect(&input1.inverse()),
            None,
            "An interval can't be intersected with its inverse"
        );
    }
}
