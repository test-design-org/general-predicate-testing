use crate::interval::{Boundary, Interval, IntervalError, MultiInterval};

pub trait Bva
where
    Self: Sized,
{
    /// Possible acceptable values except the first from the edges.
    ///
    /// Example: `[1,10)` with the precision of `0.01` will have the in of `[1.0,9.99]`
    fn calc_in(&self, precision: f32) -> MultiInterval;

    /// Possible not acceptable values except the first from the edges.
    ///
    /// Example: `[1,10)` with a precision of `0.01` will thave he out intervals of `(-Inf,0.98] [10.01,Inf)`
    fn out(&self, precision: f32) -> MultiInterval;

    /// Possible not acceptable values. This is Off+Out.
    ///
    /// Example: `[1,10)` will have the `off_out` intervals of `(-Inf,0.99] [10,Inf)`
    fn off_out(&self, precision: f32) -> MultiInterval;

    /// First acceptable values from the edges. There can be 0, 1, or 2 such points.
    ///
    /// Example: `[1,10)` with a precision of `0.01` will have the On points of `[1.0, 1.0] [9.99, 9.99]`
    fn on(&self, precision: f32) -> MultiInterval;

    /// Second acceptable values from the edges.There can be 0, 1, or 2 such points.
    ///
    /// Example: `[1,10)` with the precision of `0.01` will have the inin of `[1.01, 1.01] [9.98,9.98]`  
    fn inin(&self, precision: f32) -> MultiInterval;

    /// First not acceptable values from the edges. There can be 0, 1, or 2 such points.
    ///
    /// Example: `[1.10)` with a precision of `0.01` will have the off points of `[0.99,0.99]` and `[10.0, 10.0]`.
    fn off(&self, precision: f32) -> MultiInterval;
}

impl Bva for Interval {
    fn calc_in(&self, precision: f32) -> MultiInterval {
        // If interval.lo is f32::NEG_INFINITY this will be f32::NEG_INFINITY
        let lo = self.lo
            + if self.lo_boundary == Boundary::Open {
                1.0
            } else {
                0.0
            } * precision;

        // If interval.lo is f32::INFINITY this will be f32::INFINITY
        let hi = self.hi
            - if self.hi_boundary == Boundary::Open {
                1.0
            } else {
                0.0
            } * precision;

        if !((self.contains_point(lo) || self.lo == f32::NEG_INFINITY)
            && (self.contains_point(hi) || self.hi == f32::INFINITY))
        {
            return MultiInterval {
                intervals: Vec::new(),
            };
        }

        match Self::new_closed(lo, hi) {
            Ok(in_interval) => MultiInterval::from_interval(in_interval),
            Err(IntervalError::LoIsGreaterThanHi) => MultiInterval {
                intervals: Vec::new(),
            },
        }
    }

    fn out(&self, precision: f32) -> MultiInterval {
        let mut outs = Vec::new();

        if self.lo != f32::NEG_INFINITY {
            let out_lo = Self::new_closed(
                f32::NEG_INFINITY,
                self.lo
                    - if self.lo_boundary == Boundary::Open {
                        1.0
                    } else {
                        2.0
                    } * precision,
            )
            .expect("Should be a valid interval");

            outs.push(out_lo);
        }

        if self.hi != f32::INFINITY {
            let out_hi = Self::new_closed(
                self.hi
                    + if self.hi_boundary == Boundary::Open {
                        1.0
                    } else {
                        2.0
                    } * precision,
                f32::INFINITY,
            )
            .expect("Should be a valid interval");

            outs.push(out_hi);
        }

        MultiInterval::from_intervals(outs)
    }

    fn off_out(&self, precision: f32) -> MultiInterval {
        let mut off_outs = Vec::new();

        if self.lo != f32::NEG_INFINITY {
            let off_out_lo = Self::new_closed(
                f32::NEG_INFINITY,
                self.lo
                    - if self.lo_boundary == Boundary::Open {
                        0.0
                    } else {
                        1.0
                    } * precision,
            )
            .expect("Should be a valid interval");

            off_outs.push(off_out_lo);
        }

        if self.hi != f32::INFINITY {
            let off_out_hi = Self::new_closed(
                self.hi
                    + if self.hi_boundary == Boundary::Open {
                        0.0
                    } else {
                        1.0
                    } * precision,
                f32::INFINITY,
            )
            .expect("Should be a valid interval");

            off_outs.push(off_out_hi);
        }

        MultiInterval::from_intervals(off_outs)
    }

    fn on(&self, precision: f32) -> MultiInterval {
        let mut ons = Vec::new();

        let on_lo = self.lo
            + if self.lo_boundary == Boundary::Open {
                1.0
            } else {
                0.0
            } * precision;
        if self.contains_point(on_lo) {
            ons.push(Self::new_closed_point(on_lo));
        }

        let on_hi = self.hi
            - if self.hi_boundary == Boundary::Open {
                1.0
            } else {
                0.0
            } * precision;

        if self.contains_point(on_hi) {
            ons.push(Self::new_closed_point(on_hi));
        }

        MultiInterval::from_intervals(ons)
    }

    fn inin(&self, precision: f32) -> MultiInterval {
        // If interval.lo is f32::NEG_INFINITY this will be f32::NEG_INFINITY
        let lo = self.lo
            + if self.lo_boundary == Boundary::Open {
                2.0
            } else {
                1.0
            } * precision;

        // If interval.lo is f32::INFINITY this will be f32::INFINITY
        let hi = self.hi
            - if self.hi_boundary == Boundary::Open {
                2.0
            } else {
                1.0
            } * precision;

        match Self::new_closed(lo, hi) {
            Ok(inin) => MultiInterval::from_interval(inin),
            Err(IntervalError::LoIsGreaterThanHi) => MultiInterval {
                intervals: Vec::new(),
            },
        }
    }

    fn off(&self, precision: f32) -> MultiInterval {
        let mut offs = Vec::new();

        if self.lo != f32::NEG_INFINITY {
            let off_lo = Self::new_closed_point(
                self.lo
                    - if self.lo_boundary == Boundary::Open {
                        0.0
                    } else {
                        1.0
                    } * precision,
            );

            offs.push(off_lo);
        }

        if self.hi != f32::INFINITY {
            let off_hi = Self::new_closed_point(
                self.hi
                    + if self.hi_boundary == Boundary::Open {
                        0.0
                    } else {
                        1.0
                    } * precision,
            );

            offs.push(off_hi);
        }

        MultiInterval::from_intervals(offs)
    }
}

impl MultiInterval {
    fn bva_all_intervals(
        &self,
        precision: f32,
        bva_function: impl Fn(&Interval, f32) -> Self,
    ) -> Self {
        let bar = self
            .intervals
            .iter()
            .flat_map(|interval| bva_function(interval, precision).intervals)
            .collect();

        Self::from_intervals(bar)
    }
}

impl Bva for MultiInterval {
    fn calc_in(&self, precision: f32) -> MultiInterval {
        self.bva_all_intervals(precision, Interval::calc_in)
    }

    fn out(&self, precision: f32) -> MultiInterval {
        self.bva_all_intervals(precision, Interval::out)
    }

    fn off_out(&self, precision: f32) -> MultiInterval {
        self.bva_all_intervals(precision, Interval::off_out)
    }

    fn on(&self, precision: f32) -> MultiInterval {
        self.bva_all_intervals(precision, Interval::on)
    }

    fn inin(&self, precision: f32) -> MultiInterval {
        self.bva_all_intervals(precision, Interval::inin)
    }

    fn off(&self, precision: f32) -> MultiInterval {
        self.bva_all_intervals(precision, Interval::off)
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::Bva;
    use crate::interval::{Interval, MultiInterval};

    #[rstest]
    // Correct In calculation for the boundaries
    #[case("[1, 10]", 0.01, "[1, 10]")]
    #[case("[1, 10)", 0.01, "[1, 9.99]")]
    #[case("(1, 10)", 0.01, "[1.01, 9.99]")]
    #[case("(1, 10]", 0.01, "[1.01, 10]")]
    // Inf boundaries
    #[case::inf_left("(-Inf, 10]", 0.01, "(-Inf, 10]")]
    #[case::inf_left("(-Inf, 10)", 0.01, "(-Inf, 9.99]")]
    #[case::inf_right("[10, Inf)", 0.01, "[10, Inf)")]
    #[case::inf_right("(10, Inf)", 0.01, "[10.01, Inf)")]
    #[case::inf("(-Inf, Inf)", 0.01, "(-Inf, Inf)")]
    // Testing if in would be empty
    #[case("[1, 1]", 0.01, "[1,1]")]
    #[case("[1, 1)", 0.01, "")]
    #[case("(1, 1]", 0.01, "")]
    #[case("(1, 1)", 0.01, "")]
    #[case("[1, 1.42)", 0.42, "[1, 1]")]
    #[case("(0.5, 1]", 0.5, "[1, 1]")]
    #[case("(1, 10)", 100.0, "")]
    #[case("(1, 10]", 100.0, "")]
    #[case("[1, 10)", 100.0, "")]
    fn test_interval_in(
        #[case] input: Interval,
        #[case] precision: f32,
        #[case] expected: MultiInterval,
    ) {
        assert_eq!(input.calc_in(precision), expected);
    }

    #[rstest]
    // Correct On calculation for the boundaries
    #[case("[1, 10]", 0.01, "[1,1] [10,10]")]
    #[case("[1, 10)", 0.01, "[1,1] [9.99,9.99]")]
    #[case("(1, 10)", 0.01, "[1.01,1.01] [9.99,9.99]")]
    #[case("(1, 10]", 0.01, "[1.01,1.01] [10,10]")]
    // Inf boundaries
    #[case::inf_left("(-Inf, 10]", 0.01, "[10,10]")]
    #[case::inf_left("(-Inf, 10)", 0.01, "[9.99,9.99]")]
    #[case::inf_right("[10, Inf)", 0.01, "[10,10]")]
    #[case::inf_right("(10, Inf)", 0.01, "[10.01,10.01]")]
    #[case::inf("(-Inf, Inf)", 0.01, "")]
    // Testing if on would be empty
    #[case::single_point("[1, 1]", 0.01, "[1,1]")]
    #[case::half_open_right_empty("[1, 1)", 0.01, "")]
    #[case::half_open_left_empty("(1, 1]", 0.01, "")]
    #[case::empty("(1, 1)", 0.01, "")]
    #[case("[1, 1.42)", 0.42, "[1,1]")]
    #[case("(0.5, 1]", 0.5, "[1,1]")]
    #[case("[1, 10]", 100.0, "[1,1] [10,10]")]
    #[case("[1, 10)", 100.0, "[1,1]")]
    #[case("(1, 10]", 100.0, "[10,10]")]
    #[case("(1, 10)", 100.0, "")]
    fn test_interval_on(
        #[case] input: Interval,
        #[case] precision: f32,
        #[case] expected: MultiInterval,
    ) {
        assert_eq!(input.on(precision), expected);
    }

    // TODO: Test for interval inin
    // TODO: Test for interval off
    // TODO: Test for interval out

    // TODO: Test for multiinterval in
    // TODO: Test for multiinterval on
    // TODO: Test for multiinterval inin
    // TODO: Test for multiinterval off
    // TODO: Test for multiinterval out
}
