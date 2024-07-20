use chrono::{DateTime, Local, TimeDelta};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WorkJourneyError {
    #[error("Invalid clock-in and clock-out time boundaries.")]
    InvalidaClockBoundaries(DateTime<Local>, DateTime<Local>),
}

pub struct IncompleteWorkJourney {
    starting_time: DateTime<Local>,
    ending_time: Option<DateTime<Local>>,
}

impl IncompleteWorkJourney {
    pub fn new(current_time: DateTime<Local>) -> IncompleteWorkJourney {
        IncompleteWorkJourney {
            starting_time: current_time,
            ending_time: None,
        }
    }

    pub fn end(&mut self, current_time: DateTime<Local>) -> Result<WorkJourney, WorkJourneyError> {
        self.ending_time = Some(current_time);

        WorkJourney::new(self.starting_time, current_time)
    }
}

#[derive(Clone, Debug)]
pub struct WorkJourney {
    starting_time: DateTime<Local>,
    ending_time: DateTime<Local>,
}

impl WorkJourney {
    pub fn new(
        starting_time: DateTime<Local>,
        ending_time: DateTime<Local>,
    ) -> Result<WorkJourney, WorkJourneyError> {
        if WorkJourney::validate(starting_time, ending_time) {
            Ok(WorkJourney {
                starting_time,
                ending_time,
            })
        } else {
            Err(WorkJourneyError::InvalidaClockBoundaries(
                starting_time,
                ending_time,
            ))
        }
    }

    fn validate(starting_time: DateTime<Local>, ending_time: DateTime<Local>) -> bool {
        ending_time >= starting_time
    }

    pub fn worked_hours(&self) -> TimeDelta {
        self.ending_time - self.starting_time
    }

    pub fn get_starting_time(&self) -> DateTime<Local> {
        self.starting_time
    }

    pub fn get_ending_time(&self) -> DateTime<Local> {
        self.ending_time
    }
}

#[cfg(test)]
mod tests {
    use std::{thread::sleep, time::Duration};

    use chrono::{Local, TimeDelta};

    use crate::library::work_journey::IncompleteWorkJourney;

    #[test]
    fn basic_work_journey_initialization() {
        let now = Local::now();

        let mut new_journey = IncompleteWorkJourney::new(now);

        let now_2 = Local::now();
        let journey = new_journey.end(now_2);
        assert!(journey.is_ok())
    }

    #[test]
    fn incorrect_work_journey_initialization() {
        let now = Local::now();
        sleep(Duration::from_secs(1));
        let now_2 = Local::now();

        let mut new_journey = IncompleteWorkJourney::new(now_2);

        let journey = new_journey.end(now);
        assert!(journey.is_err())
    }

    #[test]
    fn basic_work_journey_math() {
        let now = Local::now();
        let now_plus_6 = now + TimeDelta::hours(6);

        let mut new_journey = IncompleteWorkJourney::new(now);

        let journey = new_journey.end(now_plus_6).unwrap();
        assert_eq!(now, journey.get_starting_time());
        assert_eq!(now_plus_6, journey.get_ending_time());
        assert_eq!(TimeDelta::hours(6), journey.worked_hours());
    }
}
