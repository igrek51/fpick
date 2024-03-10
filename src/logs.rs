use std::cell::RefCell;

thread_local!(static LOGS: RefCell<Vec<String>> = RefCell::new(vec![]));

pub fn log(msg: &str) {
    LOGS.with(|logs| {
        logs.borrow_mut().push(msg.to_string());
    });
}

pub fn print_logs() {
    LOGS.with(|logs| {
        let vector: Vec<String> = logs.borrow().clone();
        for log in vector {
            println!("{}", log);
        }
    });
}
