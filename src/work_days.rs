use chrono::{DateTime, Local, TimeDelta};

use crate::work_journey::WorkJourney;

const MAX_JOURNEYS_PER_DAY: usize = 5;

#[derive(Clone)]
pub struct WorkDay {
    journeys: Vec<WorkJourney>,
    worked_hours: TimeDelta,
}

impl WorkDay {
    pub fn new(journeys: &Vec<WorkJourney>) -> WorkDay {
        let worked_hours = journeys
            .iter()
            .fold(TimeDelta::zero(), |acc, item| acc + item.worked_hours());

        for (index, journey) in journeys.iter().enumerate() {
            let mut journey_reached_max = false;
            if journey.worked_hours() == TimeDelta::hours(6) {
                journey_reached_max = true;
            }

            if journey_reached_max {
                if let Some(next_journey) = journeys.get(index + 1) {
                    let inter_journey_rest =
                        next_journey.get_starting_time() - journey.get_ending_time();

                    if inter_journey_rest < TimeDelta::hours(1) {
                        println!("Inter-journey rest was violated!")
                    }
                }
            }
        }

        if worked_hours < TimeDelta::hours(6) {
            println!("Worked less than 6 hours.")
        } else if worked_hours > TimeDelta::hours(10) {
            println!("Worked more than 10 hours.")
        }

        if journeys.len() > MAX_JOURNEYS_PER_DAY {
            println!("Worked more than 5 journeys.")
        }

        WorkDay {
            journeys: journeys.to_vec(),
            worked_hours,
        }
    }

    pub fn worked_hours(&self) -> TimeDelta {
        self.worked_hours
    }

    pub fn first_clock_in(&self) -> DateTime<Local> {
        self.journeys
            .first()
            .expect(
                "Day generated without journey. There must be at least one journey in each day!",
            )
            .get_starting_time()
    }

    pub fn last_clock_out(&self) -> DateTime<Local> {
        self.journeys
            .last()
            .expect(
                "Day generated without journey. There must be at least one journey in each day!",
            )
            .get_ending_time()
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Local, TimeDelta};

    use crate::{work_days::WorkDay, work_journey::IncompleteWorkJourney};

    #[test]
    fn basic_work_day_initialization() {
        let now = Local::now();
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
        let now = Local::now();
        let now_plus_six = now + TimeDelta::hours(6);
        let now_plus_seven = now_plus_six + TimeDelta::hours(1);
        let now_plus_eight = now_plus_seven + TimeDelta::hours(1);

        let mut new_journey = IncompleteWorkJourney::new(now);
        let mut new_journey_2 = IncompleteWorkJourney::new(now_plus_seven);
        let journey = new_journey.end(now_plus_six).unwrap();
        let journey2 = new_journey_2.end(now_plus_eight).unwrap();

        let journeys = vec![journey, journey2];
        let work_day = WorkDay::new(&journeys);

        assert_eq!(now, work_day.first_clock_in());
        assert_eq!(now_plus_eight, work_day.last_clock_out());
        assert_eq!(TimeDelta::hours(7), work_day.worked_hours());
    }
}
