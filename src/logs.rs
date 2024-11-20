use lazy_static::lazy_static;
use std::sync::{Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

lazy_static! {
    static ref LOGS_MESSAGES: Mutex<Vec<String>> = Mutex::new(vec![]);
}

fn global_logs_list<'a>() -> &'a Mutex<Vec<String>> {
    &LOGS_MESSAGES
}

pub fn log(msg: &str) {
    let mut guard: MutexGuard<'_, Vec<String>> = global_logs_list().lock().unwrap();
    let time_str = current_time_str();
    guard.push(format!("[{}] {}", time_str, msg));
}

pub fn print_logs() {
    let guard: MutexGuard<'_, Vec<String>> = global_logs_list().lock().unwrap();
    let vector: Vec<String> = guard.clone();
    for log in vector {
        eprintln!("{}", log);
    }
}

pub fn current_time_str() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let seconds_since_epoch = now.as_secs();
    let hours = (seconds_since_epoch / 3600) % 24;
    let minutes = (seconds_since_epoch / 60) % 60;
    let seconds = seconds_since_epoch % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}
