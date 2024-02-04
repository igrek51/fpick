use anyhow::Error;

pub fn contextualized_error(error: &Error) -> String {
    error
        .chain()
        .map(|e| e.to_string())
        .collect::<Vec<String>>()
        .join(": ")
}
