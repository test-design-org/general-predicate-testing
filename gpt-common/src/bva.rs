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
    /// Example: `[1,10)` with a precision of `0.01` will the out intervals of `(-Inf,0.98] [10.01,Inf)`
    fn out(&self, precision: f32) -> MultiInterval;

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

        match Interval::new_closed(lo, hi) {
            Ok(in_interval) => MultiInterval::from_interval(in_interval),
            Err(IntervalError::LoIsGreaterThanHi) => MultiInterval {
                intervals: Vec::new(),
            },
        }
    }

    fn out(&self, precision: f32) -> MultiInterval {
        let mut outs = Vec::new();

        if self.lo != f32::NEG_INFINITY {
            let out_lo = Interval::new_closed(
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
            let out_hi = Interval::new_closed(
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

        MultiInterval { intervals: outs }
    }

    fn on(&self, precision: f32) -> MultiInterval {
        let mut ons = Vec::new();

        if self.lo != f32::NEG_INFINITY {
            let on_lo = Interval::new_closed_point(
                self.lo
                    + if self.lo_boundary == Boundary::Open {
                        1.0
                    } else {
                        0.0
                    } * precision,
            );

            ons.push(on_lo);
        }

        if self.hi != f32::INFINITY {
            let on_hi = Interval::new_closed_point(
                self.hi
                    - if self.hi_boundary == Boundary::Open {
                        1.0
                    } else {
                        0.0
                    } * precision,
            );

            ons.push(on_hi);
        }

        MultiInterval { intervals: ons }
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

        match Interval::new_closed(lo, hi) {
            Ok(inin) => MultiInterval::from_interval(inin),
            Err(IntervalError::LoIsGreaterThanHi) => MultiInterval {
                intervals: Vec::new(),
            },
        }
    }

    fn off(&self, precision: f32) -> MultiInterval {
        let mut offs = Vec::new();

        if self.lo != f32::NEG_INFINITY {
            let off_lo = Interval::new_closed_point(
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
            let off_hi = Interval::new_closed_point(
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

impl Bva for MultiInterval {
    fn calc_in(&self, precision: f32) -> MultiInterval {
        let ins = self
            .intervals
            .iter()
            .flat_map(|interval| interval.calc_in(precision).intervals)
            .collect();

        MultiInterval::from_intervals(ins)
    }

    fn out(&self, precision: f32) -> MultiInterval {
        let outs = self
            .intervals
            .iter()
            .flat_map(|interval| interval.out(precision).intervals)
            .collect();

        MultiInterval::from_intervals(outs)
    }

    fn on(&self, precision: f32) -> MultiInterval {
        let ons = self
            .intervals
            .iter()
            .flat_map(|interval| interval.on(precision).intervals)
            .collect();

        MultiInterval::from_intervals(ons)
    }

    fn inin(&self, precision: f32) -> MultiInterval {
        let inins = self
            .intervals
            .iter()
            .flat_map(|interval| interval.inin(precision).intervals)
            .collect();

        MultiInterval::from_intervals(inins)
    }

    fn off(&self, precision: f32) -> MultiInterval {
        let offs = self
            .intervals
            .iter()
            .flat_map(|interval| interval.off(precision).intervals)
            .collect();

        MultiInterval::from_intervals(offs)
    }
}
