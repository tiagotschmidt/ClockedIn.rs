use std::ops::AddAssign;

use chrono::TimeDelta;

#[derive(Clone, Copy)]
enum HourState {
    Debt,
    Credit,
}

#[derive(Clone, Copy)]
pub struct DeltaHours {
    original_delta: TimeDelta,
    unsigned_delta: TimeDelta,
    state: HourState,
}

impl DeltaHours {
    pub fn new(original_delta: TimeDelta) -> DeltaHours {
        let mut state = HourState::Credit;
        if original_delta >= TimeDelta::zero() {
            state = HourState::Debt;
        }

        let unsigned_delta = if original_delta > TimeDelta::zero() {
            original_delta
        } else {
            -original_delta
        };

        DeltaHours {
            original_delta,
            unsigned_delta,
            state,
        }
    }
}

impl AddAssign for DeltaHours {
    fn add_assign(&mut self, rhs: Self) {
        self.original_delta += rhs.original_delta

        let mut state = HourState::Credit;
        if self.original_delta >= TimeDelta::zero() {
            state = HourState::Debt;
        }

        let unsigned_delta = if self.original_delta > TimeDelta::zero() {
            self.original_delta
        } else {
            -self.original_delta
        };

        self.unsigned_delta = unsigned_delta;
    }
}
