use super::{
    long_term_registry::LongTermRegistry, work_days::WorkDay, work_journey::IncompleteWorkJourney,
    work_week::WorkWeek,
};

struct ClockedInService {
    long_term_registry: LongTermRegistry,
    current_work_journey: Option<IncompleteWorkJourney>,
    current_work_day: Option<WorkDay>,
    current_work_week: Option<WorkWeek>,
}

impl ClockedInService {
    pub fn new() -> ClockedInService {
        let long_term_registry = LongTermRegistry::new();
        let current_work_journey = None;
        let current_work_day = None;
        let current_work_week = None;

        ClockedInService {
            long_term_registry,
            current_work_journey,
            current_work_day,
            current_work_week,
        }
    }
}
