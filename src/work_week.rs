use std::num::TryFromIntError;

use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};

use crate::{delta_hours::DeltaHours, work_days::WorkDay};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum InterDayViolation {
    InterDayRestViolation,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct WorkWeek {
    workdays: Vec<WorkDay>,
    violations: Option<InterDayViolation>,
}

impl WorkWeek {
    pub fn new() -> WorkWeek {
        let workdays: Vec<WorkDay> = Vec::with_capacity(5);
        let violations = None;

        WorkWeek {
            workdays,
            violations,
        }
    }

    pub fn append_day(&mut self, day: WorkDay) {
        if self.workdays.len() < 5 {
            self.workdays.push(day)
        }

        for (index, day) in self.workdays.iter().enumerate() {
            if let Some(next_day) = self.workdays.get(index + 1) {
                let next_day_first_clock_in = next_day.first_clock_in();
                let this_day_first_clock_in = day.last_clock_out();

                if next_day_first_clock_in - this_day_first_clock_in < TimeDelta::hours(11) {
                    println!("Inter-day rest was violated!");
                    self.violations = Some(InterDayViolation::InterDayRestViolation);
                }
            }
        }
    }

    pub fn worked_hours(&self) -> i64 {
        self.workdays.iter().fold(0, |mut acc, item| {
            acc += item.worked_hours();
            acc
        })
    }

    fn days_worked(&self) -> usize {
        self.workdays.len()
    }

    fn expected_hours(&self) -> Result<i64, TryFromIntError> {
        let hours = (self.days_worked() * 8).try_into()?;
        Ok(TimeDelta::hours(hours).num_seconds())
    }

    pub fn worked_delta(&self) -> Result<DeltaHours, TryFromIntError> {
        let current_delta_time = self.expected_hours()? - self.worked_hours();
        Ok(DeltaHours::new(current_delta_time))
    }

    pub fn last_clock_out_last_day_in_week(&self) -> Option<DateTime<Utc>> {
        self.workdays.last().map(|item| item.last_clock_out())
    }
}

impl Default for WorkWeek {
    fn default() -> Self {
        Self::new()
    }
}
