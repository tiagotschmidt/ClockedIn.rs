use std::{
    fs::OpenOptions,
    io::{Read, Write},
};

use chrono::{DateTime, Datelike, NaiveDate, TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const LONG_TERM_REGISTRY_STATE_FILE_NAME: &str = "long_term_registry_state.json";
const EXPECTED_WORK_JOURNEY_TIME_DELTA: TimeDelta = TimeDelta::hours(8);
const EXPECTED_OVERTIME_WORK_JOURNEY_TIME_DELTA: TimeDelta = TimeDelta::hours(2);

use crate::work_days::MAX_HOURS_PER_JOURNEY;

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
    #[error("Error during opening of long term state file.")]
    LongTermRegistryOpenError,
    #[error("ClockIn day in the last day of the last week of registry.")]
    CurrentClockInDaySameAsLastWeekInRegistry,
    #[error("ClockIn day in the last day of the current work week.")]
    CurrentClockInDaySameAsCurrentWorkWeek,
}

#[derive(Serialize, Deserialize)]
pub struct ClockedInService {
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
        if let Some(last_clock_out) = self.long_term_registry.last_clock_out_last_week() {
            let same_year = starting_time.year() == last_clock_out.year();
            let same_month = starting_time.month() == last_clock_out.month();
            let same_day = starting_time.day() == last_clock_out.day();

            if same_year && same_month && same_day {
                return Err(ClockedInServiceError::CurrentClockInDaySameAsLastWeekInRegistry);
            }
        }

        if let Some(last_week) = &self.current_work_week {
            if let Some(last_clock_out) = last_week.last_clock_out_last_day_in_week() {
                let same_year = starting_time.year() == last_clock_out.year();
                let same_month = starting_time.month() == last_clock_out.month();
                let same_day = starting_time.day() == last_clock_out.day();

                if same_year && same_month && same_day {
                    return Err(ClockedInServiceError::CurrentClockInDaySameAsCurrentWorkWeek);
                }
            }
        }

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
                self.current_work_journey = None;

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
                let mut new_work_week = WorkWeek::new();
                new_work_week.append_day(finished_work_day);
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

    pub fn worked_hours_today(&self) -> TimeDelta {
        self.current_work_day
            .iter()
            .fold(TimeDelta::zero(), |acc, item| acc + item.worked_hours())
    }

    pub fn worked_hours_this_week(&self) -> Vec<(NaiveDate, TimeDelta)> {
        let mut return_vec = Vec::new();

        if let Some(week) = &self.current_work_week {
            for day in &week.workdays {
                return_vec.push((
                    day.first_clock_in().date_naive(),
                    TimeDelta::seconds(day.worked_hours()),
                ))
            }
        }

        return_vec
    }

    pub fn recommended_journey(&self) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
        let worked_hours_today = self.worked_hours_today();
        let remaining_hours = EXPECTED_WORK_JOURNEY_TIME_DELTA - worked_hours_today;

        if remaining_hours < TimeDelta::zero() {
            return None;
        }

        if let Some(current_journey) = &self.current_work_journey {
            let preview_journey_end = if remaining_hours > MAX_HOURS_PER_JOURNEY {
                let current_journey_start = current_journey.starting_time;
                let preview_journey_end = current_journey_start + TimeDelta::hours(6);

                Some((preview_journey_end, preview_journey_end))
            } else {
                let current_journey_start = current_journey.starting_time;
                let preview_journey_end = current_journey_start + remaining_hours;

                if remaining_hours + EXPECTED_OVERTIME_WORK_JOURNEY_TIME_DELTA
                    < MAX_HOURS_PER_JOURNEY
                {
                    Some((
                        preview_journey_end,
                        preview_journey_end + EXPECTED_OVERTIME_WORK_JOURNEY_TIME_DELTA,
                    ))
                } else {
                    Some((preview_journey_end, preview_journey_end))
                }
            };
            return preview_journey_end;
        } else {
            return None;
        }
    }

    fn serialize_to_json(&self) -> Result<String, ClockedInServiceError> {
        serde_json::to_string(&self).map_err(|_err| ClockedInServiceError::SerializationError)
    }

    fn deserialize_from_json(serialized: String) -> Result<Self, ClockedInServiceError> {
        serde_json::from_str(&serialized).map_err(|_err| ClockedInServiceError::SerializationError)
    }

    pub fn save_state(&self) -> Result<(), ClockedInServiceError> {
        let mut file = open_or_create_long_term_registry_file_to_write()?;

        let _ = file.write_all(self.serialize_to_json()?.as_bytes());
        Ok(())
    }

    pub fn read_state() -> Result<ClockedInService, ClockedInServiceError> {
        let mut file = match open_long_term_registry_file_to_read() {
            Ok(file) => file,
            Err(_err) => open_or_create_long_term_registry_file_to_write()?,
        };

        let mut serialized_state = String::new();
        let _ = file.read_to_string(&mut serialized_state);

        ClockedInService::deserialize_from_json(serialized_state)
    }
}

impl Default for ClockedInService {
    fn default() -> Self {
        Self::new()
    }
}

fn open_or_create_long_term_registry_file_to_write() -> Result<std::fs::File, ClockedInServiceError>
{
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(LONG_TERM_REGISTRY_STATE_FILE_NAME)
        .map_err(|_| ClockedInServiceError::LongTermRegistryOpenError)?;
    Ok(file)
}

fn open_long_term_registry_file_to_read() -> Result<std::fs::File, ClockedInServiceError> {
    let file = OpenOptions::new()
        .read(true)
        .open(LONG_TERM_REGISTRY_STATE_FILE_NAME)
        .map_err(|_| ClockedInServiceError::LongTermRegistryOpenError)?;
    Ok(file)
}
