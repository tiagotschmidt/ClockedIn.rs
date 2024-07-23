use std::{fmt::Display, ops::AddAssign};

use colored::Colorize;

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

impl Display for DeltaHours {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let current_hours = self.unsigned_delta / 60 / 60;
        let current_minutes = (self.unsigned_delta - current_hours * 60 * 60) / 60;
        let current_seconds =
            (self.unsigned_delta - current_hours * 60 * 60) - current_minutes * 60;

        let temp = match self.state {
            HourState::Debt => format!(
                "{} {} {}, {} {}, {} {}",
                "Missing".red(),
                current_hours.to_string().red().bold(),
                "hours".red(),
                current_minutes.to_string().red().bold(),
                "minutes".red(),
                current_seconds.to_string().red().bold(),
                "seconds.".red(),
            ),
            HourState::Credit if self.original_delta != 0 => format!(
                "{} {} {}, {} {}, {} {}",
                "Exceeding".green(),
                current_hours.to_string().green().bold(),
                "hours".green(),
                current_minutes.to_string().green().bold(),
                "minutes".green(),
                current_seconds.to_string().green().bold(),
                "seconds.".green(),
            ),
            HourState::Credit => "Delta is zero".to_string(),
        };

        write!(f, "{}", temp)
    }
}

impl Default for DeltaHours {
    fn default() -> Self {
        Self {
            original_delta: Default::default(),
            unsigned_delta: Default::default(),
            state: HourState::Credit,
        }
    }
}
