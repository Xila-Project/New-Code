#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use log::{error, info};

pub fn Initialize() {
    info!("Initializing...");

    info!("Initialize task module");

    if let Err(Error) = Task::Initialize() {
        error!("Failed to initialize the task module: {:?}", Error);
        std::process::exit(1);
    }

    info!("Initialization user module");

    if let Err(Error) = Users::Initialize() {
        error!("Failed to initialize the user module: {:?}", Error);
        std::process::exit(1);
    }

    info!("Initialize file system module");

    if let Err(Error) = File_system::Initialize() {
        error!("Failed to initialize the file system module: {:?}", Error);
        std::process::exit(1);
    }

    info!("Initialize virtual machine module");

    info!("Initialization complete");
}
