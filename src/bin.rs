use std::{env, io};

use clockedin_utils::{
    clockedin_service::{ClockedInService, ClockedInServiceError},
    delta_hours::DeltaHours,
};

#[repr(u8)]
pub enum MainProgramOptions {
    ClockIn = b'0',
    ClockOut = b'1',
    ClockOutAndEndDay = b'2',
    ClockOutAndEndWeek = b'3',
    Invalid,
}

impl MainProgramOptions {
    pub fn from(c: char) -> MainProgramOptions {
        match c {
            '0' => Self::ClockIn,
            '1' => Self::ClockOut,
            '2' => Self::ClockOutAndEndDay,
            '3' => Self::ClockOutAndEndWeek,
            _ => Self::Invalid,
        }
    }
}

fn main() {
    let raw_args: Vec<String> = env::args().collect();
    let args: Vec<String> = raw_args.into_iter().skip(2).collect();
    let mut clockedin_service = prologue();
    let current_delta = match clockedin_service.worked_delta_until_today() {
        Ok(delta) => delta,
        Err(err) => {
            panic_epilogue(&clockedin_service, err);
            DeltaHours::new(0)
        }
    };

    if !args.is_empty() {
    } else {
        loop {
            println!("##############################################################");
            println!("\tClockedIn Terminal Version:");
            println!("Current Delta (until today): {}", current_delta);
            println!("0. ClockIn");
            println!("1. ClockOut");
            println!("2. ClockOut and end Day");
            println!("3. CLockOut and end Week");
            println!("##############################################################");
            let mut buffer = String::new();
            io::stdin().read_line(&mut buffer).unwrap();

            let now = chrono::offset::Utc::now();
            match buffer.chars().next() {
                Some(current_char) => match MainProgramOptions::from(current_char) {
                    MainProgramOptions::ClockIn => {
                        match clockedin_service.clock_in(now) {
                            Ok(_) => (),
                            Err(err) => {
                                panic_epilogue(&clockedin_service, err);
                            }
                        }
                        break;
                    }
                    MainProgramOptions::ClockOut => {
                        match clockedin_service.clock_out(now) {
                            Ok(_) => (),
                            Err(err) => panic_epilogue(&clockedin_service, err),
                        }
                        break;
                    }
                    MainProgramOptions::ClockOutAndEndDay => {
                        match clockedin_service.clock_out_and_end_work_day(now) {
                            Ok(_) => (),
                            Err(err) => panic_epilogue(&clockedin_service, err),
                        }
                        break;
                    }
                    MainProgramOptions::ClockOutAndEndWeek => {
                        match clockedin_service.clock_out_and_end_work_week(now) {
                            Ok(_) => (),
                            Err(err) => panic_epilogue(&clockedin_service, err),
                        }
                        break;
                    }
                    MainProgramOptions::Invalid => continue,
                },
                None => todo!(),
            }
        }
    }

    epilogue(&clockedin_service);
}

fn panic_epilogue(clockedin_service: &ClockedInService, err: ClockedInServiceError) {
    epilogue(clockedin_service);
    panic!("Error occurred: {}", err)
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

fn epilogue(current_service: &ClockedInService) {
    match current_service.save_state() {
        Ok(()) => (),
        Err(clockedin_service_error) => match clockedin_service_error {
            ClockedInServiceError::SerializationError => panic!("Error at serialization!"),
            ClockedInServiceError::LongTermRegistryOpenError => {
                panic!("Error at registry opening!");
            }
            err => panic!("Impossible to happen: {}", err),
        },
    }
}
