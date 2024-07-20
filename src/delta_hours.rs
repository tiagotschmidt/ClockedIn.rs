use std::ops::AddAssign;

#[derive(Clone, Copy)]
enum HourState {
    Debt,
    Credit,
}

#[derive(Clone, Copy)]
pub struct DeltaHours {
    original_delta: i64,
    unsigned_delta: i64,
    state: HourState,
}

impl DeltaHours {
    pub fn new(original_delta: i64) -> DeltaHours {
        let mut state = HourState::Credit;
        if original_delta >= 0 {
            state = HourState::Debt;
        }

        let unsigned_delta = if original_delta > 0 {
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
        self.original_delta += rhs.original_delta;

        if self.original_delta >= 0 {
            self.state = HourState::Debt;
        }

        let unsigned_delta = if self.original_delta > 0 {
            self.original_delta
        } else {
            -self.original_delta
        };

        self.unsigned_delta = unsigned_delta;
    }
}
