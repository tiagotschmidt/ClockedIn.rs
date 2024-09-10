use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};

use crate::work_journey::WorkJourney;

const MAX_JOURNEYS_PER_DAY: usize = 5;
pub const MAX_HOURS_PER_JOURNEY: TimeDelta = TimeDelta::hours(6);

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum IntraDayViolation {
    ExceddedMaxHours,
    MissingHours,
    ViolatedInterJourneyRest,
    ExceddedMaxJourneys,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkDay {
    journeys: Vec<WorkJourney>,
    worked_seconds: i64,
    violations: Vec<IntraDayViolation>,
}

impl WorkDay {
    pub fn new(journeys: &[WorkJourney]) -> WorkDay {
        let mut day_violations = Vec::new();

        let worked_hours = journeys
            .iter()
            .fold(TimeDelta::zero(), |acc, item| acc + item.worked_hours())
            .num_seconds();

        if worked_hours >= TimeDelta::hours(6).num_seconds() {
            let mut has_found_a_sufficient_rest = false;
            for (index, journey) in journeys.iter().enumerate() {
                if let Some(next_journey) = journeys.get(index + 1) {
                    let inter_journey_rest =
                        next_journey.get_starting_time() - journey.get_ending_time();

                    if inter_journey_rest >= TimeDelta::hours(1) {
                        has_found_a_sufficient_rest = true;
                    }
                }
            }
            if !has_found_a_sufficient_rest {
                day_violations.push(IntraDayViolation::ViolatedInterJourneyRest);
            }
        }

        if worked_hours < TimeDelta::hours(6).num_seconds() {
            day_violations.push(IntraDayViolation::MissingHours);
        } else if worked_hours > TimeDelta::hours(10).num_seconds() {
            day_violations.push(IntraDayViolation::ExceddedMaxHours);
        }

        if journeys.len() > MAX_JOURNEYS_PER_DAY {
            day_violations.push(IntraDayViolation::ExceddedMaxJourneys);
        }

        WorkDay {
            journeys: journeys.to_vec(),
            worked_seconds: worked_hours,
            violations: day_violations,
        }
    }

    pub fn worked_hours(&self) -> i64 {
        self.worked_seconds
    }

    pub fn first_clock_in(&self) -> DateTime<Utc> {
        self.journeys
            .first()
            .expect(
                "Day generated without journey. There must be at least one journey in each day!",
            )
            .get_starting_time()
    }

    pub fn last_clock_out(&self) -> DateTime<Utc> {
        self.journeys
            .last()
            .expect(
                "Day generated without journey. There must be at least one journey in each day!",
            )
            .get_ending_time()
    }

    pub fn get_violations(&self) -> Vec<IntraDayViolation> {
        self.violations.clone()
    }
}

#[cfg(test)]
pub mod tests {
    use chrono::{TimeDelta, Utc};

    use crate::{work_days::WorkDay, work_journey::IncompleteWorkJourney};

    #[test]
    fn basic_work_day_initialization() {
        let now = Utc::now();
        let now_plus_six = now + TimeDelta::hours(6);
        let now_plus_seven = now + TimeDelta::hours(1);
        let now_plus_eight = now + TimeDelta::hours(1);

        let mut new_journey = IncompleteWorkJourney::new(now);
        let mut new_journey_2 = IncompleteWorkJourney::new(now_plus_seven);
        let journey = new_journey.end(now_plus_six).unwrap();
        let journey2 = new_journey_2.end(now_plus_eight).unwrap();

        let journeys = vec![journey, journey2];
        let _work_day = WorkDay::new(&journeys);
    }

    #[test]
    fn basic_work_day_math() {
        let (now, now_plus_eight, work_day) = initialize_mock_day();

        assert_eq!(now, work_day.first_clock_in());
        assert_eq!(now_plus_eight, work_day.last_clock_out());
        assert_eq!(TimeDelta::hours(7).num_seconds(), work_day.worked_hours());
    }

    #[test]
    fn missing_hours_violation_check() {
        let (_now, _now_plus_eightt, work_day) = initialize_missing_hours_violated_mock_day();

        assert!(work_day
            .violations
            .contains(&crate::work_days::IntraDayViolation::MissingHours));

        assert_eq!(TimeDelta::hours(5).num_seconds(), work_day.worked_hours());
    }

    #[test]
    fn inter_journey_violation_check() {
        let (_now, _now_plus_eight, work_day) = inter_journey_violated_mock_day();

        assert!(work_day
            .violations
            .contains(&crate::work_days::IntraDayViolation::ViolatedInterJourneyRest));

        assert_eq!(TimeDelta::hours(7).num_seconds(), work_day.worked_hours());
    }

    #[test]
    fn excedded_hours_violation_check() {
        let (_now, _now_plus_eight, work_day) = excedded_hours_violated_mock_day();

        assert!(work_day
            .violations
            .contains(&crate::work_days::IntraDayViolation::ExceddedMaxHours));

        assert_eq!(TimeDelta::hours(11).num_seconds(), work_day.worked_hours());
    }

    pub fn initialize_mock_day() -> (chrono::DateTime<Utc>, chrono::DateTime<Utc>, WorkDay) {
        let now = Utc::now();
        let now_plus_six = now + TimeDelta::hours(6);
        let now_plus_seven = now_plus_six + TimeDelta::hours(1);
        let now_plus_eight = now_plus_seven + TimeDelta::hours(1);

        let mut new_journey = IncompleteWorkJourney::new(now);
        let mut new_journey_2 = IncompleteWorkJourney::new(now_plus_seven);
        let journey = new_journey.end(now_plus_six).unwrap();
        let journey2 = new_journey_2.end(now_plus_eight).unwrap();

        let journeys = vec![journey, journey2];
        let work_day = WorkDay::new(&journeys);
        (now, now_plus_eight, work_day)
    }

    fn initialize_missing_hours_violated_mock_day(
    ) -> (chrono::DateTime<Utc>, chrono::DateTime<Utc>, WorkDay) {
        let now = Utc::now();
        let now_plus_four = now + TimeDelta::hours(4);
        let now_plus_four_and_a_half = now_plus_four + TimeDelta::minutes(30);
        let now_plus_five_and_a_half = now_plus_four_and_a_half + TimeDelta::hours(1);

        let mut new_journey = IncompleteWorkJourney::new(now);
        let mut new_journey_2 = IncompleteWorkJourney::new(now_plus_four_and_a_half);
        let journey = new_journey.end(now_plus_four).unwrap();
        let journey2 = new_journey_2.end(now_plus_five_and_a_half).unwrap();

        let journeys = vec![journey, journey2];
        let work_day = WorkDay::new(&journeys);
        (now, now_plus_five_and_a_half, work_day)
    }

    fn excedded_hours_violated_mock_day() -> (chrono::DateTime<Utc>, chrono::DateTime<Utc>, WorkDay)
    {
        let now = Utc::now();
        let now_plus_six = now + TimeDelta::hours(6);
        let now_plus_seven = now_plus_six + TimeDelta::hours(1);
        let now_plus_twelve = now_plus_seven + TimeDelta::hours(5);

        let mut new_journey = IncompleteWorkJourney::new(now);
        let mut new_journey_2 = IncompleteWorkJourney::new(now_plus_seven);
        let journey = new_journey.end(now_plus_six).unwrap();
        let journey2 = new_journey_2.end(now_plus_twelve).unwrap();

        let journeys = vec![journey, journey2];
        let work_day = WorkDay::new(&journeys);
        (now, now_plus_twelve, work_day)
    }

    fn inter_journey_violated_mock_day() -> (chrono::DateTime<Utc>, chrono::DateTime<Utc>, WorkDay)
    {
        let now = Utc::now();
        let now_plus_six = now + TimeDelta::hours(6);
        let now_plus_six_and_a_half = now_plus_six + TimeDelta::minutes(30);
        let now_plus_seven_and_a_half = now_plus_six_and_a_half + TimeDelta::hours(1);

        let mut new_journey = IncompleteWorkJourney::new(now);
        let mut new_journey_2 = IncompleteWorkJourney::new(now_plus_six_and_a_half);
        let journey = new_journey.end(now_plus_six).unwrap();
        let journey2 = new_journey_2.end(now_plus_seven_and_a_half).unwrap();

        let journeys = vec![journey, journey2];
        let work_day = WorkDay::new(&journeys);
        (now, now_plus_seven_and_a_half, work_day)
    }
}
