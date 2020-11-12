use std::cell::BorrowError;
use std::ffi::CString;
use std::ffi::NulError;
use std::fmt;
use std::os::raw::{c_char, c_int};
use std::ptr;

use serde_json::Error as JSONError;
use ssi::error::Error as SSIError;
use std::str::Utf8Error;

static UNKNOWN_ERROR: &str = "Unable to create error string\0";

use std::cell::RefCell;
thread_local! {
    pub static LAST_ERROR: RefCell<Option<(i32, CString)>> = RefCell::new(None);
}

#[derive(Debug)]
pub enum Error {
    SSI(ssi::error::Error),
    Null(NulError),
    Utf8(Utf8Error),
    Borrow(BorrowError),

    #[doc(hidden)]
    __Nonexhaustive,
}

impl Error {
    pub fn stash(self) {
        LAST_ERROR.with(|stash| {
            stash.replace(Some((
                self.get_code(),
                CString::new(self.to_string()).unwrap(),
            )))
        });
    }

    fn get_code(&self) -> c_int {
        // TODO: try to give each individual error its own number
        match self {
            Error::SSI(_) => 1,
            Error::Null(_) => 2,
            Error::Utf8(_) => 3,
            _ => -1,
        }
    }
}

#[no_mangle]
pub extern "C" fn didkit_error_message() -> *const c_char {
    LAST_ERROR.with(|error| match error.try_borrow() {
        Ok(maybe_err_ref) => match &*maybe_err_ref {
            Some(err) => err.1.as_ptr() as *const c_char,
            None => ptr::null(),
        },
        Err(_) => UNKNOWN_ERROR.as_ptr() as *const c_char,
    })
}

#[no_mangle]
pub extern "C" fn didkit_error_code() -> c_int {
    LAST_ERROR.with(|error| match error.try_borrow() {
        Ok(maybe_err_ref) => match &*maybe_err_ref {
            Some(err) => err.0,
            None => 0,
        },
        Err(err) => Error::from(err).get_code(),
    })
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::SSI(e) => e.fmt(f),
            Error::Null(e) => e.fmt(f),
            Error::Utf8(e) => e.fmt(f),
            _ => unreachable!(),
        }
    }
}

impl From<SSIError> for Error {
    fn from(err: SSIError) -> Error {
        Error::SSI(err)
    }
}

impl From<JSONError> for Error {
    fn from(err: JSONError) -> Error {
        Error::SSI(SSIError::from(err))
    }
}

impl From<NulError> for Error {
    fn from(err: NulError) -> Error {
        Error::Null(err)
    }
}

impl From<Utf8Error> for Error {
    fn from(err: Utf8Error) -> Error {
        Error::Utf8(err)
    }
}

impl From<BorrowError> for Error {
    fn from(err: BorrowError) -> Error {
        Error::Borrow(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors() {
        use crate::c::didkit_vc_issue_presentation;
        use std::ffi::CStr;
        let presentation = CString::new("{}").unwrap().as_ptr();
        let options = CString::new("{}").unwrap().as_ptr();
        let key = CString::new("{}").unwrap().as_ptr();
        let vp = didkit_vc_issue_presentation(presentation, options, key);
        assert_eq!(vp, ptr::null());
        let msg = unsafe { CStr::from_ptr(didkit_error_message()) }
            .to_str()
            .unwrap();
        let code = didkit_error_code();
        assert_ne!(code, 0);
        println!("code: {:?} msg: {:?}", code, msg);
    }
}