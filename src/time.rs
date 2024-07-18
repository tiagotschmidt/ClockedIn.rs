use chrono::{DateTime, Local, TimeZone};

struct WorkJourney {
    starting_time: DateTime<Local>,
    ending_time: DateTime<Local>,
}

impl WorkJourney {
    fn new(starting_time: DateTime<Local>, ending_time: DateTime<Local>) -> WorkJourney {
        WorkJourney {
            starting_time,
            ending_time,
        }
    }
}

pub struct WorkDay {
    journeys: Vec<WorkJourney>,
}

pub struct WorkWeek {
    workdays: Vec<Option<WorkDay>>,
}

impl WorkWeek {
    fn new() -> WorkWeek {
        let workdays: Vec<Option<WorkDay>> = Vec::with_capacity(5);

        WorkWeek { workdays }
    }
}
