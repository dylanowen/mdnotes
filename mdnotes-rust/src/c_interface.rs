use core::ptr;
use std::ffi::CStr;
use std::os::raw::c_char;

use crate::MdNotesRuntime;

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub extern "C" fn md_notes_runtime_new() -> *mut MdNotesRuntime {
    match MdNotesRuntime::new() {
        Ok(runtime) => Box::into_raw(Box::new(runtime)),
        Err(e) => {
            error!("Error creating MdNotes Runtime{}", e);

            ptr::null_mut()
        }
    }
}

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn md_notes_runtime_free(ptr: *mut MdNotesRuntime) {
    if ptr.is_null() {
        return;
    }

    Box::from_raw(ptr);
}

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn md_notes_runtime_server_port(ptr: *mut MdNotesRuntime) -> u16 {
    let runtime = &mut *ptr;

    runtime.server_port()
}

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn md_notes_runtime_open_notes(
    ptr: *mut MdNotesRuntime,
    raw_path: *const c_char,
) -> u8 {
    let runtime = &mut *ptr;
    let path = CStr::from_ptr(raw_path).to_str().unwrap().to_string();

    runtime.open_notes(path.into())
}

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn md_notes_runtime_close_notes(ptr: *mut MdNotesRuntime, notes_id: u8) {
    let runtime = &mut *ptr;

    runtime.close_notes(notes_id);
}
