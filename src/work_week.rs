use chrono::TimeDelta;

use crate::work_days::WorkDay;

pub struct WorkWeek {
    workdays: Vec<Option<WorkDay>>,
}

impl WorkWeek {
    pub fn new() -> WorkWeek {
        let workdays: Vec<Option<WorkDay>> = Vec::with_capacity(5);

        WorkWeek { workdays }
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
                        println!("Inter-day rest was violated!")
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
                    acc = acc + work_day.worked_hours()
                }
                acc
            })
    }

    fn days_worked(&self) -> usize {
        self.workdays.iter().filter(|item| item.is_some()).count()
    }

    fn expected_hours(&self) -> TimeDelta {
        TimeDelta::hours(
            (self.days_worked() * 8)
                .try_into()
                .expect("Error during number conversion!"),
        )
    }

    pub fn worked_delta(&self) -> TimeDelta {
        self.expected_hours() - self.worked_hours()
    }
}
