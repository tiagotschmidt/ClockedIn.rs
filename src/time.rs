use chrono::TimeDelta;

enum HourState {
    Debt,
    Credit,
}

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
