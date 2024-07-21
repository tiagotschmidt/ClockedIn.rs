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
