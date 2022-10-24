#[derive(Debug, PartialEq)]
pub enum Openness {
    Open,
    Closed,
}

#[derive(Debug, PartialEq)]
struct OneInterval {
    lo_openness: Openness,
    lo: f32,
    hi: f32,
    hi_openness: Openness,
}

#[derive(Debug, PartialEq)]
pub struct Interval {
    intervals: Vec<OneInterval>,
}

impl Interval {
    pub fn new(lo_openness: Openness, lo: f32, hi: f32, hi_openness: Openness) -> Interval {
        Interval {
            intervals: vec![OneInterval {
                lo_openness,
                lo,
                hi,
                hi_openness,
            }],
        }
    }
}
