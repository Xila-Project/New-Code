use std::{ffi::CStr, ptr::null_mut};

use wamr_rust_sdk::{module::Module, sys::wasm_runtime_set_wasi_args_ex};
use File_system::Unique_file_identifier_type;

use crate::{Error_type, Result_type, Runtime::Runtime_type};

pub struct Module_type {
    Module: Module,
    _Environment_variables_raw: Vec<*const i8>,
}

unsafe impl Send for Module_type {}

const Directory_paths: [&CStr; 1] = [unsafe { CStr::from_bytes_with_nul_unchecked(b"/\0") }];

const Directory_paths_raw: [*const i8; 1] = [Directory_paths[0].as_ptr()];

impl Module_type {
    pub fn From_buffer(
        Runtime: &Runtime_type,
        Buffer: Vec<u8>,
        Name: &str,
        Standard_in: Unique_file_identifier_type,
        Standard_out: Unique_file_identifier_type,
        Standard_error: Unique_file_identifier_type,
    ) -> Result_type<Self> {
        // - Environment variables.
        let Task_instance = Task::Get_instance();

        let Task = Task_instance
            .Get_current_task_identifier()
            .map_err(Error_type::Failed_to_get_task_informations)?;

        let mut Environment_variables_raw: Vec<*const i8> = Task_instance
            .Get_environment_variables(Task)
            .map_err(Error_type::Failed_to_get_task_informations)?
            .into_iter()
            .map(|x| x.Get_raw().as_ptr())
            .collect();

        let Environment_variables_raw_pointer = Environment_variables_raw.as_mut_ptr();

        let Environment_variables_length = Environment_variables_raw.len();

        // - Create the module.
        let Module = Module_type {
            Module: Module::from_vec(Runtime.Get_inner_reference(), Buffer, Name)?,
            _Environment_variables_raw: Environment_variables_raw,
        };

        let Standard_in = Standard_in.Into_inner() as u64;
        let Standard_out = Standard_out.Into_inner() as u64;
        let Standard_error = Standard_error.Into_inner() as u64;

        // - Set WASI arguments.
        unsafe {
            wasm_runtime_set_wasi_args_ex(
                Module.Module.get_inner_module(),
                Directory_paths_raw.as_ptr() as *mut *const i8,
                Directory_paths_raw.len() as u32,
                null_mut(),
                0,
                Environment_variables_raw_pointer,
                Environment_variables_length as u32,
                null_mut(),
                0,
                std::mem::transmute::<u64, i64>(Standard_in),
                std::mem::transmute::<u64, i64>(Standard_out),
                std::mem::transmute::<u64, i64>(Standard_error),
            )
        }

        Ok(Module)
    }

    pub(crate) fn Get_inner_reference(&self) -> &Module {
        &self.Module
    }
}
