use Time::Get_instance;

pub type Xila_time_type = u64;

pub type Xila_time_clock_identifier_type = usize;

/// Retrieve the current time since the system startup in microseconds.
///
/// # Returns
///
/// The current time since the system startup in microseconds.
#[no_mangle]
pub extern "C" fn Xila_time_get_time_since_startup_microseconds() -> u64 {
    Get_instance()
        .Get_current_time_since_startup()
        .unwrap_or_default()
        .As_microseconds() as u64
}

#[no_mangle]
pub extern "C" fn Xila_time_get_cpu() -> u64 {
    todo!()
}

/// Retrieve the current time since the system startup in milliseconds.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[no_mangle]
pub unsafe extern "C" fn Xila_time_get_resolution(
    _Clock_identifier: Xila_time_clock_identifier_type,
    _Resolution: *mut Xila_time_type,
) -> u32 {
    todo!()
}

/// Retrieve the current time since the system startup in milliseconds.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[no_mangle]
pub extern "C" fn Xila_time_get_time(
    _Clock_identifier: Xila_time_clock_identifier_type,
    _Precision: u64,
    _Time: *mut Xila_time_type,
) -> u32 {
    todo!()
}
