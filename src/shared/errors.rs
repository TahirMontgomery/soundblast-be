use std::io::Error;

pub fn extract_error(err: Error) -> String {
    err.to_string()
}
