use std::num::TryFromIntError;

use chrono::TimeDelta;

use crate::{library::delta_hours::DeltaHours, library::work_days::WorkDay};

pub enum InterDayViolation {
    InterDayRestViolation,
}

pub struct WorkWeek {
    workdays: Vec<Option<WorkDay>>,
    violations: Option<InterDayViolation>,
}

impl WorkWeek {
    pub fn new() -> WorkWeek {
        let workdays: Vec<Option<WorkDay>> = Vec::with_capacity(5);
        let violations = None;

        WorkWeek {
            workdays,
            violations,
        }
    }

    pub fn append_day(&mut self, day: WorkDay) {
        if self.workdays.len() < 5 {
            self.workdays.push(Some(day))
        }

        for (index, day) in self.workdays.iter().enumerate() {
            if let Some(Some(next_day)) = self.workdays.get(index + 1) {
                if let Some(day) = day {
                    let next_day_first_clock_in = next_day.first_clock_in();
                    let this_day_first_clock_in = day.last_clock_out();

                    if next_day_first_clock_in - this_day_first_clock_in < TimeDelta::hours(11) {
                        println!("Inter-day rest was violated!");
                        self.violations = Some(InterDayViolation::InterDayRestViolation);
                    }
                }
            }
        }
    }

    pub fn worked_hours(&self) -> TimeDelta {
        self.workdays
            .iter()
            .fold(TimeDelta::zero(), |mut acc, item| {
                if let Some(work_day) = item {
                    acc += work_day.worked_hours()
                }
                acc
            })
    }

    fn days_worked(&self) -> usize {
        self.workdays.iter().filter(|item| item.is_some()).count()
    }

    fn expected_hours(&self) -> Result<TimeDelta, TryFromIntError> {
        let hours = (self.days_worked() * 8).try_into()?;
        Ok(TimeDelta::hours(hours))
    }

    pub fn worked_delta(&self) -> Result<DeltaHours, TryFromIntError> {
        let current_delta_time = self.expected_hours()? - self.worked_hours();
        Ok(DeltaHours::new(current_delta_time))
    }
}

impl Default for WorkWeek {
    fn default() -> Self {
        Self::new()
    }
}
