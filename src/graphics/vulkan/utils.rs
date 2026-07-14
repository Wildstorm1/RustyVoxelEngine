use std::ffi::{CString};

/*
 * Converts a rust string to a c-string, converting the error to another
 * error type
 */
pub fn c_string_or_error<E>(string: String, error: E) -> Result<CString, E> {
    match CString::new(string) {
        Ok(c_string) => Ok(c_string),
        Err(e) => {
            println!("Non-terminating nul byte found: {}", e);
            return Err(error);
        }
    }
}