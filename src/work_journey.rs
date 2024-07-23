use chrono::{serde::ts_seconds, serde::ts_seconds_option, DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WorkJourneyError {
    #[error("Invalid clock-in and clock-out time boundaries.")]
    InvalidClockBoundaries(DateTime<Utc>, DateTime<Utc>),
}

#[derive(Serialize, Deserialize)]
pub struct IncompleteWorkJourney {
    #[serde(with = "ts_seconds")]
    pub starting_time: DateTime<Utc>,
    #[serde(with = "ts_seconds_option")]
    ending_time: Option<DateTime<Utc>>,
}

impl IncompleteWorkJourney {
    pub fn new(current_time: DateTime<Utc>) -> IncompleteWorkJourney {
        IncompleteWorkJourney {
            starting_time: current_time,
            ending_time: None,
        }
    }

    pub fn end(&mut self, current_time: DateTime<Utc>) -> Result<WorkJourney, WorkJourneyError> {
        self.ending_time = Some(current_time);

        WorkJourney::new(self.starting_time, current_time)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkJourney {
    starting_time: DateTime<Utc>,
    ending_time: DateTime<Utc>,
}

impl WorkJourney {
    pub fn new(
        starting_time: DateTime<Utc>,
        ending_time: DateTime<Utc>,
    ) -> Result<WorkJourney, WorkJourneyError> {
        if WorkJourney::validate(starting_time, ending_time) {
            Ok(WorkJourney {
                starting_time,
                ending_time,
            })
        } else {
            Err(WorkJourneyError::InvalidClockBoundaries(
                starting_time,
                ending_time,
            ))
        }
    }

    fn validate(starting_time: DateTime<Utc>, ending_time: DateTime<Utc>) -> bool {
        ending_time >= starting_time
    }

    pub fn worked_hours(&self) -> TimeDelta {
        self.ending_time - self.starting_time
    }

    pub fn get_starting_time(&self) -> DateTime<Utc> {
        self.starting_time
    }

    pub fn get_ending_time(&self) -> DateTime<Utc> {
        self.ending_time
    }
}

#[cfg(test)]
mod tests {
    use std::{thread::sleep, time::Duration};

    use chrono::{TimeDelta, Utc};

    use crate::work_journey::IncompleteWorkJourney;

    #[test]
    fn basic_work_journey_initialization() {
        let now = Utc::now();

        let mut new_journey = IncompleteWorkJourney::new(now);

        let now_2 = Utc::now();
        let journey = new_journey.end(now_2);
        assert!(journey.is_ok())
    }

    #[test]
    fn incorrect_work_journey_initialization() {
        let now = Utc::now();
        sleep(Duration::from_secs(1));
        let now_2 = Utc::now();

        let mut new_journey = IncompleteWorkJourney::new(now_2);

        let journey = new_journey.end(now);
        assert!(journey.is_err())
    }

    #[test]
    fn basic_work_journey_math() {
        let now = Utc::now();
        let now_plus_6 = now + TimeDelta::hours(6);

        let mut new_journey = IncompleteWorkJourney::new(now);

        let journey = new_journey.end(now_plus_6).unwrap();
        assert_eq!(now, journey.get_starting_time());
        assert_eq!(now_plus_6, journey.get_ending_time());
        assert_eq!(TimeDelta::hours(6), journey.worked_hours());
    }
}
