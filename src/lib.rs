use std::ffi::CStr;
use std::os::raw::c_char;

extern "C" {
    fn cadical_version() -> *const c_char;
}

pub fn version() -> String {
    let s = unsafe { CStr::from_ptr(cadical_version()) };
    s.to_string_lossy().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        assert_eq!(version(), "1.2.1".to_string());
    }
}
