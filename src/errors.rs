use nom::ErrorKind;
use std::sync::Mutex;

macro_rules! err_str(
    ($msg:expr) => {{
        use nom::ErrorKind;
        use errors::def_error;
        lazy_static! {
            static ref ERR: ErrorKind = def_error($msg);
        }
        ERR.clone()
    }}
);

pub fn def_error(msg: &'static str) -> ErrorKind {
    let mut errors = ERRORS.lock().unwrap();
    let n = errors.len();
    errors.push(msg);
    ErrorKind::Custom(n as u32)
}

pub fn get_error(n: u32) -> Option<String> {
    match ERRORS.lock() {
        Ok(e) => e.get(n as usize).map(|s| s.to_string()),
        Err(_) => None,
    }
}

lazy_static! {
    static ref ERRORS: Mutex<Vec<&'static str>> = Mutex::new(Vec::new());
}
