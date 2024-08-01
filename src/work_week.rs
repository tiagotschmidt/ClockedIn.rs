use std::num::TryFromIntError;

use chrono::{DateTime, TimeDelta, Utc};
use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::{delta_hours::DeltaHours, work_days::WorkDay};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum InterDayViolation {
    InterDayRestViolation,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct WorkWeek {
    pub workdays: Vec<WorkDay>,
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

    pub fn append_day(&mut self, day: &WorkDay) {
        if self.workdays.len() < 5 {
            self.workdays.push(day.clone())
        }

        for (index, day) in self.workdays.iter().enumerate() {
            if let Some(next_day) = self.workdays.get(index + 1) {
                let next_day_first_clock_in = next_day.first_clock_in();
                let this_day_first_clock_in = day.last_clock_out();

                if next_day_first_clock_in - this_day_first_clock_in < TimeDelta::hours(11) {
                    println!(
                        "{}",
                        "Inter-day rest was violated!"
                            .red()
                            .on_bright_white()
                            .bold()
                    );
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

    pub fn worked_delta(&self) -> Result<DeltaHours, TryFromIntError> {
        let current_delta_time = self.expected_hours()? - self.worked_hours();
        Ok(DeltaHours::new(current_delta_time))
    }

    pub fn last_clock_out_last_day_in_week(&self) -> Option<DateTime<Utc>> {
        self.workdays.last().map(|item| item.last_clock_out())
    }

    fn days_worked(&self) -> usize {
        self.workdays.len()
    }

    fn expected_hours(&self) -> Result<i64, TryFromIntError> {
        let hours = (self.days_worked() * 8).try_into()?;
        Ok(TimeDelta::hours(hours).num_seconds())
    }
}

impl Default for WorkWeek {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
pub mod tests {
    use chrono::TimeDelta;

    use crate::{delta_hours::DeltaHours, work_days::tests::initialize_mock_day};

    use super::WorkWeek;

    #[test]
    fn basic_work_week_initialization() {
        let _mock_week = intialize_mock_week();
    }

    #[test]
    fn basic_work_week_math() {
        let mock_week = intialize_mock_week();
        let (_now, _now_plus_eightt, work_day_five) = initialize_mock_day();

        assert_eq!(
            TimeDelta::hours(5 * 7).num_seconds(),
            mock_week.worked_hours()
        );
        assert!(
            work_day_five.last_clock_out() - mock_week.last_clock_out_last_day_in_week().unwrap()
                < TimeDelta::seconds(1)
        );
        assert_eq!(
            DeltaHours::new(TimeDelta::hours(5).num_seconds()),
            mock_week.worked_delta().unwrap()
        )
    }

    #[test]
    fn missing_hours_violation_check() {
        let mock_week = intialize_mock_week();

        assert!(mock_week.violations.is_some());
    }

    pub fn intialize_mock_week() -> WorkWeek {
        let (_now, _now_plus_eightt, work_day_one) = initialize_mock_day();
        let (_now, _now_plus_eightt, work_day_two) = initialize_mock_day();
        let (_now, _now_plus_eightt, work_day_three) = initialize_mock_day();
        let (_now, _now_plus_eightt, work_day_four) = initialize_mock_day();
        let (_now, _now_plus_eightt, work_day_five) = initialize_mock_day();

        let mut _new_work_week = WorkWeek::new();
        _new_work_week.append_day(&work_day_one);
        _new_work_week.append_day(&work_day_two);
        _new_work_week.append_day(&work_day_three);
        _new_work_week.append_day(&work_day_four);
        _new_work_week.append_day(&work_day_five);
        _new_work_week
    }
}
