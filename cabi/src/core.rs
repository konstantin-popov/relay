use std::mem;
use std::ptr;
use std::str;
use std::slice;
use std::ffi::CStr;
use std::os::raw::c_char;

use utils::{set_panic_hook, LAST_ERROR};

/// Represents a string.
#[repr(C)]
pub struct SmithStr {
    pub data: *mut c_char,
    pub len: usize,
    pub owned: bool,
}

impl Default for SmithStr {
    fn default() -> SmithStr {
        SmithStr {
            data: ptr::null_mut(),
            len: 0,
            owned: false,
        }
    }
}

impl SmithStr {
    pub fn new(s: &str) -> SmithStr {
        SmithStr {
            data: s.as_ptr() as *mut c_char,
            len: s.len(),
            owned: false,
        }
    }

    pub fn from_string(mut s: String) -> SmithStr {
        s.shrink_to_fit();
        let rv = SmithStr {
            data: s.as_ptr() as *mut c_char,
            len: s.len(),
            owned: true,
        };
        mem::forget(s);
        rv
    }

    pub unsafe fn free(&mut self) {
        if self.owned {
            String::from_raw_parts(self.data as *mut _, self.len, self.len);
            self.data = ptr::null_mut();
            self.len = 0;
            self.owned = false;
        }
    }

    pub fn as_str(&self) -> &str {
        unsafe {
            str::from_utf8_unchecked(slice::from_raw_parts(
                self.data as *const _, self.len))
        }
    }
}

/// Initializes the library
#[no_mangle]
pub unsafe extern "C" fn smith_init() {
    set_panic_hook();
}

/// Returns the last error code.
///
/// If there is no error, 0 is returned.
#[no_mangle]
pub unsafe extern "C" fn smith_err_failed() -> bool {
    LAST_ERROR.with(|e| {
        if let Some(..) = *e.borrow() {
            true
        } else {
            false
        }
    })
}

/// Returns the last error message.
///
/// If there is no error an empty string is returned.  This allocates new memory
/// that needs to be freed with `smith_str_free`.
#[no_mangle]
pub unsafe extern "C" fn smith_err_get_last_message() -> SmithStr {
    use std::fmt::Write;
    use std::error::Error;
    LAST_ERROR.with(|e| {
        if let Some(ref err) = *e.borrow() {
            let mut msg = err.to_string();
            for cause in err.causes().skip(1) {
                write!(&mut msg, "\n  caused by: {}", cause).ok();
            }
            SmithStr::from_string(msg)
        } else {
            Default::default()
        }
    })
}

/// Returns the panic information as string.
#[no_mangle]
pub unsafe extern "C" fn smith_err_get_backtrace() -> SmithStr {
    LAST_ERROR.with(|e| {
        if let Some(ref error) = *e.borrow() {
            let backtrace = error.backtrace().to_string();
            if !backtrace.is_empty() {
                use std::fmt::Write;
                let mut out = String::new();
                write!(&mut out, "stacktrace: {}", backtrace).ok();
                SmithStr::from_string(out)
            } else {
                Default::default()
            }
        } else {
            Default::default()
        }
    })
}

/// Clears the last error.
#[no_mangle]
pub unsafe extern "C" fn smith_err_clear() {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = None;
    });
}

ffi_fn! {
    /// Creates a smith str from a c string.
    ///
    /// This sets the string to owned.  In case it's not owned you either have
    /// to make sure you are not freeing the memory or you need to set the
    /// owned flag to false.
    unsafe fn smith_str_from_cstr(s: *const c_char) -> Result<SmithStr> {
        let s = CStr::from_ptr(s).to_str()?;
        Ok(SmithStr {
            data: s.as_ptr() as *mut _,
            len: s.len(),
            owned: true,
        })
    }
}

/// Frees a smith str.
///
/// If the string is marked as not owned then this function does not
/// do anything.
#[no_mangle]
pub unsafe extern "C" fn smith_str_free(s: *mut SmithStr) {
    if !s.is_null() {
        (*s).free()
    }
}
