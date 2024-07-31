use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::num::TryFromIntError;
use thiserror::Error;

use crate::{delta_hours::DeltaHours, work_week::WorkWeek};

#[derive(Error, Debug)]
pub enum LongTermRegistryError {
    #[error("Empty history.")]
    EmptyHistory,
    #[error("Error during int conversion {0}")]
    IntConversionError(TryFromIntError),
}

#[derive(Serialize, Deserialize)]
pub struct LongTermRegistry {
    pub history: Vec<WorkWeek>,
}

impl LongTermRegistry {
    pub fn new() -> LongTermRegistry {
        let history = Vec::new();

        LongTermRegistry { history }
    }

    pub fn worked_hours(&self) -> i64 {
        self.history.iter().fold(0, |mut acc, item| {
            acc += item.worked_hours();
            acc
        })
    }

    pub fn worked_delta(&self) -> Result<DeltaHours, LongTermRegistryError> {
        let mut current_delta = if let Some(work_week) = self.history.first() {
            work_week
                .worked_delta()
                .map_err(LongTermRegistryError::IntConversionError)?
        } else {
            DeltaHours::default()
        };

        for week in self.history.iter().skip(1) {
            current_delta += week
                .worked_delta()
                .map_err(LongTermRegistryError::IntConversionError)?
        }

        Ok(current_delta)
    }

    pub fn last_clock_out_last_week(&self) -> Option<DateTime<Utc>> {
        self.history
            .last()
            .and_then(|item| item.last_clock_out_last_day_in_week())
    }
}

impl Default for LongTermRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeDelta;

    use crate::{delta_hours::DeltaHours, work_week::tests::intialize_mock_week};

    use super::LongTermRegistry;

    #[test]
    fn basic_long_term_registry_initialization() {
        let _mock_long_term_registry = initialize_mock_long_term_registry();
    }

    #[test]
    fn basic_work_week_math() {
        let _mock_long_term_registry = initialize_mock_long_term_registry();
        let week4 = intialize_mock_week();

        assert_eq!(
            TimeDelta::hours(4 * 5 * 7).num_seconds(),
            _mock_long_term_registry.worked_hours()
        );
        assert!(
            week4.last_clock_out_last_day_in_week().unwrap()
                - _mock_long_term_registry.last_clock_out_last_week().unwrap()
                < TimeDelta::seconds(1)
        );
        assert_eq!(
            DeltaHours::new(TimeDelta::hours(20).num_seconds()),
            _mock_long_term_registry.worked_delta().unwrap()
        )
    }

    fn initialize_mock_long_term_registry() -> LongTermRegistry {
        let mut long_term_registry = LongTermRegistry::new();
        let week1 = intialize_mock_week();
        let week2 = intialize_mock_week();
        let week3 = intialize_mock_week();
        let week4 = intialize_mock_week();

        long_term_registry.history.push(week1);
        long_term_registry.history.push(week2);
        long_term_registry.history.push(week3);
        long_term_registry.history.push(week4);
        long_term_registry
    }
}
