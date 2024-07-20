use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::error;
use thiserror::Error;

use super::{
    delta_hours::DeltaHours,
    long_term_registry::{LongTermRegistry, LongTermRegistryError},
    work_days::WorkDay,
    work_journey::{IncompleteWorkJourney, WorkJourney, WorkJourneyError},
    work_week::WorkWeek,
};

#[derive(Error, Debug)]
pub enum ClockedInServiceError {
    #[error("A work journey already has been started at {0}.")]
    WorkJourneyAlreadyInProgess(DateTime<Utc>),
    #[error("Error during work journey ending: {0}.")]
    WorkJourneyEndingError(WorkJourneyError),
    #[error("No work journey in progress.")]
    NoneCurrentWorkJourney(),
    #[error("Error during long term registry acess {0}")]
    LongTermRegistryError(LongTermRegistryError),
    #[error("Error during serialization for general state.")]
    SerializationError,
}

#[derive(Serialize, Deserialize)]
struct ClockedInService {
    long_term_registry: LongTermRegistry,
    current_work_journey: Option<IncompleteWorkJourney>,
    current_work_day: Vec<WorkJourney>,
    current_work_week: Option<WorkWeek>,
}

impl ClockedInService {
    pub fn new() -> ClockedInService {
        let long_term_registry = LongTermRegistry::new();
        let current_work_journey = None;
        let current_work_day = Vec::new();
        let current_work_week = None;

        ClockedInService {
            long_term_registry,
            current_work_journey,
            current_work_day,
            current_work_week,
        }
    }

    pub fn clock_in(&mut self, starting_time: DateTime<Utc>) -> Result<(), ClockedInServiceError> {
        match &self.current_work_journey {
            Some(current_work_journey) => Err(ClockedInServiceError::WorkJourneyAlreadyInProgess(
                current_work_journey.starting_time,
            )),
            None => {
                let new_work_journey = IncompleteWorkJourney::new(starting_time);
                self.current_work_journey = Some(new_work_journey);
                Ok(())
            }
        }
    }

    pub fn clock_out(&mut self, ending_time: DateTime<Utc>) -> Result<(), ClockedInServiceError> {
        match &mut self.current_work_journey {
            Some(current_work_journey) => {
                let finished_journey = current_work_journey
                    .end(ending_time)
                    .map_err(ClockedInServiceError::WorkJourneyEndingError)?;

                self.current_work_day.push(finished_journey);

                Ok(())
            }
            None => Err(ClockedInServiceError::NoneCurrentWorkJourney()),
        }
    }

    pub fn clock_out_and_end_work_day(
        &mut self,
        ending_time: DateTime<Utc>,
    ) -> Result<(), ClockedInServiceError> {
        self.clock_out(ending_time)?;

        let finished_work_day = WorkDay::new(&self.current_work_day);
        self.current_work_day = Vec::new();

        match &mut self.current_work_week {
            Some(current_work_week) => current_work_week.append_day(finished_work_day),
            None => {
                let new_work_week = WorkWeek::new();
                self.current_work_week = Some(new_work_week);
            }
        }
        Ok(())
    }

    pub fn clock_out_and_end_work_week(
        &mut self,
        ending_time: DateTime<Utc>,
    ) -> Result<(), ClockedInServiceError> {
        self.clock_out_and_end_work_day(ending_time)?;

        if let Some(current_work_week) = &self.current_work_week {
            self.long_term_registry
                .history
                .push(current_work_week.clone());

            let new_work_week = WorkWeek::new();
            self.current_work_week = Some(new_work_week);
        }
        Ok(())
    }

    pub fn worked_delta_until_today(&self) -> Result<DeltaHours, ClockedInServiceError> {
        let mut long_time_registry_delta = self
            .long_term_registry
            .worked_delta()
            .map_err(ClockedInServiceError::LongTermRegistryError)?;

        if let Some(current_work_week) = &self.current_work_week {
            long_time_registry_delta += current_work_week
                .worked_delta()
                .map_err(LongTermRegistryError::IntConversionError)
                .map_err(ClockedInServiceError::LongTermRegistryError)?;
        };

        Ok(long_time_registry_delta)
    }

    pub fn serialize_to_json(&self) -> Result<String, ClockedInServiceError> {
        serde_json::to_string(&self).map_err(|_err| ClockedInServiceError::SerializationError)
    }
}
