//! Stable C ABI entry points for desktop integrations.

use libc::c_uint;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum synchole_storage_mode {
    SyncholeStorageBinary = 0,
    SyncholeStorageSql = 1,
    SyncholeStorageDocument = 2,
}

#[no_mangle]
pub extern "C" fn synchole_version_major() -> c_uint {
    0
}

#[no_mangle]
pub extern "C" fn synchole_storage_mode_is_valid(mode: c_uint) -> bool {
    matches!(mode, 0..=2)
}
