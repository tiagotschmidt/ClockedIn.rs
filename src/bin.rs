use clockedin_utils::clockedin_service::{ClockedInService, ClockedInServiceError};

fn main() {
    let clockedin_service = prologue();
    println!("Hello world!");
}

fn prologue() -> ClockedInService {
    match ClockedInService::read_state() {
        Ok(long_term_registry_state) => long_term_registry_state,
        Err(clockedin_service_error) => match clockedin_service_error {
            ClockedInServiceError::SerializationError => ClockedInService::new(),
            ClockedInServiceError::LongTermRegistryOpenError => {
                panic!("Error at registry opening!");
            }
            err => panic!("Impossible to happen: {}", err),
        },
    }
}

fn epilogue(current_service_state)