use std::borrow::BorrowMut;
use std::sync::{Mutex, MutexGuard, Once};

static mut LOGS_MESSAGES: Option<Mutex<Vec<String>>> = None;
static INIT_LOGS: Once = Once::new();

fn global_logs_list<'a>() -> &'a Mutex<Vec<String>> {
    INIT_LOGS.call_once(|| {
        // Since this access is inside a call_once, before any other accesses, it is safe
        unsafe {
            *LOGS_MESSAGES.borrow_mut() = Some(Mutex::new(vec![]));
        }
    });
    // As long as this function is the only place with access to the static variable,
    // giving out a read-only borrow here is safe because it is guaranteed no more mutable
    // references will exist at this point or in the future.
    unsafe { LOGS_MESSAGES.as_ref().unwrap() }
}

pub fn log(msg: &str) {
    let mut guard: MutexGuard<'_, Vec<String>> = global_logs_list().lock().unwrap();
    let vector: &mut Vec<String> = &mut *guard;
    vector.push(msg.to_string());
}

pub fn print_logs() {
    let guard: MutexGuard<'_, Vec<String>> = global_logs_list().lock().unwrap();
    let vector: Vec<String> = guard.clone();
    for log in vector {
        eprintln!("{}", log);
    }
}
