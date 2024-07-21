use std::{env, io};

use chrono::{DateTime, NaiveDateTime, NaiveTime, Utc};
use clockedin_utils::clockedin_service::{ClockedInService, ClockedInServiceError};

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
    let args: Vec<String> = raw_args.into_iter().skip(1).collect();
    let mut clockedin_service = prologue();
    let current_delta = match clockedin_service.worked_delta_until_today() {
        Ok(delta) => delta,
        Err(err) => {
            panic!("Error occurred: {}", err);
        }
    };

    if !args.is_empty() {
        let now_date = chrono::offset::Utc::now().date_naive();
        let command = args.first().expect("Impossible to happen.");
        let time_string = args
            .get(1)
            .expect("Incorrect program usage. Program usage example: ./clockin in 10:35");
        let clockin_time = NaiveTime::parse_from_str(time_string, "%H:%M")
            .expect("Error occurred during time parsing");
        let clockin_date_time: DateTime<Utc> =
            DateTime::from_naive_utc_and_offset(NaiveDateTime::new(now_date, clockin_time), Utc);

        if command == "in" {
            match clockedin_service.clock_in(clockin_date_time) {
                Ok(_) => (),
                Err(err) => {
                    panic_epilogue(&clockedin_service, err);
                }
            }
        } else if command == "out" {
            match clockedin_service.clock_out(clockin_date_time) {
                Ok(_) => (),
                Err(err) => panic_epilogue(&clockedin_service, err),
            }
        } else if command == "out_day" {
            match clockedin_service.clock_out_and_end_work_day(clockin_date_time) {
                Ok(_) => (),
                Err(err) => panic_epilogue(&clockedin_service, err),
            }
        } else if command == "out_week" {
            match clockedin_service.clock_out_and_end_work_week(clockin_date_time) {
                Ok(_) => (),
                Err(err) => panic_epilogue(&clockedin_service, err),
            }
        }

        println!("##############################################################");
        println!("\tClockedIn");
        println!("Current Delta (until today): {}", current_delta);
        println!("##############################################################");
    } else {
        loop {
            println!("##############################################################");
            println!("\tClockedIn Terminal Version");
            println!("Current Delta (until today): {}", current_delta);
            println!("0. ClockIn");
            println!("1. ClockOut");
            println!("2. ClockOut and end Day");
            println!("3. ClockOut and end Week");
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
