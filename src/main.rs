#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

#[cfg(target_os = "espidf")]
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

mod File_system;
use File_system::*;

mod Log;

use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
};

use wamr_sys::*;

#[no_mangle]
pub extern "C" fn carresser_le_capitaine() -> i32 {
    println!("Carresser le capitaine !!!");
    18
}

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    #[cfg(target_os = "espidf")]
    esp_idf_sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    #[cfg(target_os = "espidf")]
    esp_idf_svc::log::EspLogger::initialize_default();

    let mut wasm_module_bytes = include_bytes!("fuck.wasm");

    let wasm_module_bytes = include_bytes! {"fuck.wasm"};
    let mut wasm_module_bytes_2: [u8; 100_000] = [0; 100_000];
    wasm_module_bytes_2[..wasm_module_bytes.len()].clone_from_slice(wasm_module_bytes);

    let mut error_buf: Vec<i8> = vec![32; 128];
    const STACK_SIZE: u32 = 8092;

    unsafe {
        if !wasm_runtime_init() {
            Error!("Failed to init WASM runtime");
        }


        let symbol = CString::new("carresser_le_capitaine").unwrap();
        let signature = CString::new("()i").unwrap(); 
        

        let mut ntv: [NativeSymbol; 1] = [{ 
            {
                NativeSymbol {
                    symbol: symbol.as_ptr() as *const c_char,
                    func_ptr: carresser_le_capitaine as *mut std::os::raw::c_void,
                    signature: signature.as_ptr() as *const c_char,
                    attachment: std::ptr::null_mut()
                }
            }
        }];

        let module_name = CString::new("env").unwrap();

        if !wasm_runtime_register_natives(module_name.as_ptr() as *const c_char, ntv.as_mut_ptr(), ntv.len() as u32) {
            Error!("Failed to register FFI");
        }

        let module = wasm_runtime_load(
            wasm_module_bytes_2.as_mut_ptr(),
            wasm_module_bytes.len() as u32,
            error_buf.as_mut_ptr(),
            error_buf.len() as u32,
        );

        if module.is_null() {
            panic!(
                "Module is null: {}",
                CStr::from_ptr(error_buf.as_ptr()).to_string_lossy()
            );
        }

        let args = vec![CString::new("test").unwrap()];

        let c_args = args
            .iter()
            .map(|arg| arg.as_ptr())
            .collect::<Vec<*const c_char>>();

        let null_ptr = 0 as *mut *const i8;

        wasm_runtime_set_wasi_args(
            module,
            null_ptr,
            0,
            null_ptr,
            0,
            null_ptr,
            0,
            c_args.as_ptr() as *mut *mut i8,
            c_args.len() as i32,
        );

        let module_inst = wasm_runtime_instantiate(
            module,
            STACK_SIZE,
            8 * 1024,
            error_buf.as_mut_ptr(),
            error_buf.len() as u32,
        );

        if module_inst.is_null() {
            panic!(
                "Instatiated module is null: {}",
                CStr::from_ptr(error_buf.as_ptr()).to_string_lossy()
            );
        }

        let exec_env = wasm_runtime_create_exec_env(module_inst, STACK_SIZE);

        if exec_env.is_null() {
            panic!(
                "Exec env is null: {}",
                CStr::from_ptr(error_buf.as_ptr()).to_string_lossy()
            );
        }

        let start_func = wasm_runtime_lookup_wasi_start_function(module_inst);

        if start_func.is_null() {
            panic!("start func is null");
        }

        let mut results: Vec<wasm_val_t> = vec![];
        let mut args: Vec<wasm_val_t> = vec![];

        if wasm_runtime_call_wasm_a(
            exec_env,
            start_func,
            results.len() as u32,
            results.as_mut_ptr(),
            args.len() as u32,
            args.as_mut_ptr(),
        ) {
            println!("success");
        }
    }
}

/*
use embedded_sdmmc::*;
use esp_idf_hal::{
    delay,
    gpio::*,
    peripherals::Peripherals,
    prelude::*,
    spi::{config::{Duplex, DriverConfig}, *},
};



const FILE_NAME: &'static str = "logs.txt";

mod File_system;



pub struct SdMmcClock;

impl TimeSource for SdMmcClock {
    fn get_timestamp(&self) -> Timestamp {
        Timestamp {
            year_since_1970: 0,
            zero_indexed_month: 0,
            zero_indexed_day: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
        }
    }
}


fn get_partition() -> Result<(), ()>
{
    const PARTITION1_START: usize = 446;
    const PARTITION2_START: usize = PARTITION1_START + PARTITION_INFO_LENGTH;
    const PARTITION3_START: usize = PARTITION2_START + PARTITION_INFO_LENGTH;
    const PARTITION4_START: usize = PARTITION3_START + PARTITION_INFO_LENGTH;
    const FOOTER_START: usize = 510;
    const FOOTER_VALUE: u16 = 0xAA55;
    const PARTITION_INFO_LENGTH: usize = 16;
    const PARTITION_INFO_STATUS_INDEX: usize = 0;
    const PARTITION_INFO_TYPE_INDEX: usize = 4;
    const PARTITION_INFO_LBA_START_INDEX: usize = 8;
    const PARTITION_INFO_NUM_BLOCKS_INDEX: usize = 12;

    if self.open_volumes.is_full() {
        return Err(());
    }

    for v in self.open_volumes.iter() {
        if v.idx == volume_idx {
            return Err(());
        }
    }

    let (part_type, lba_start, num_blocks) = {
        let mut blocks = [Block::new()];
        self.block_device
            .read(&mut blocks, BlockIdx(0), "read_mbr")
            .map_err(Error::DeviceError)?;
        let block = &blocks[0];
        // We only support Master Boot Record (MBR) partitioned cards, not
        // GUID Partition Table (GPT)
        if LittleEndian::read_u16(&block[FOOTER_START..FOOTER_START + 2]) != FOOTER_VALUE {
            return Err(Error::FormatError("Invalid MBR signature"));
        }
        let partition = match volume_idx {
            VolumeIdx(0) => {
                &block[PARTITION1_START..(PARTITION1_START + PARTITION_INFO_LENGTH)]
            }
            VolumeIdx(1) => {
                &block[PARTITION2_START..(PARTITION2_START + PARTITION_INFO_LENGTH)]
            }
            VolumeIdx(2) => {
                &block[PARTITION3_START..(PARTITION3_START + PARTITION_INFO_LENGTH)]
            }
            VolumeIdx(3) => {
                &block[PARTITION4_START..(PARTITION4_START + PARTITION_INFO_LENGTH)]
            }
            _ => {
                return Err(Error::NoSuchVolume);
            }
        };
        // Only 0x80 and 0x00 are valid (bootable, and non-bootable)
        if (partition[PARTITION_INFO_STATUS_INDEX] & 0x7F) != 0x00 {
            return Err(Error::FormatError("Invalid partition status"));
        }
        let lba_start = LittleEndian::read_u32(
            &partition[PARTITION_INFO_LBA_START_INDEX..(PARTITION_INFO_LBA_START_INDEX + 4)],
        );
        let num_blocks = LittleEndian::read_u32(
            &partition[PARTITION_INFO_NUM_BLOCKS_INDEX..(PARTITION_INFO_NUM_BLOCKS_INDEX + 4)],
        );
        (
            partition[PARTITION_INFO_TYPE_INDEX],
            BlockIdx(lba_start),
            BlockCount(num_blocks),
        )
    };
    match part_type {
        PARTITION_ID_FAT32_CHS_LBA
        | PARTITION_ID_FAT32_LBA
        | PARTITION_ID_FAT16_LBA
        | PARTITION_ID_FAT16 => {
            let volume = fat::parse_volume(&self.block_device, lba_start, num_blocks)?;
            let id = Volume(self.id_generator.get());
            let info = VolumeInfo {
                volume_id: id,
                idx: volume_idx,
                volume_type: volume,
            };
            // We already checked for space
            self.open_volumes.push(info).unwrap();
            Ok(id)
        }
        _ => Err(Error::FormatError("Partition type not supported")),
    }
}

fn main() {

    log::info!("Starting 6-spi\nThis application writes to a micro-SD card\n");

    let peripherals = Peripherals::take().unwrap();

    let Driver_config = DriverConfig::new().dma(Dma::Disabled);

    // SPI sd card init
    let driver = SpiDriver::new(
        peripherals.spi2,
        peripherals.pins.gpio12,       // SCK
        peripherals.pins.gpio11,       // PICO
        Some(peripherals.pins.gpio13), // POCI
        &Driver_config,
    )
    .unwrap();

    let mut spi_config = SpiConfig::new().baudrate(40.MHz().into()).duplex(Duplex::Full);
    let spi = SpiDeviceDriver::new(driver, Option::<Gpio10>::None, &spi_config).unwrap();
    let sdmmc_cs = PinDriver::output(peripherals.pins.gpio10).unwrap();

    let mut sdcard = SdCard::new(spi, sdmmc_cs, Delayer{});

    //let mut sdmmc_spi = embedded_sdmmc::SdMmcSpi::new(spi, sdmmc_cs);

    log::info!("Card size {} bytes", sdcard.num_bytes().unwrap());

    sdcard.write(blocks, start_block_idx)


    let mut volume_mgr = VolumeManager::new(sdcard, SdMmcClock{});

    log::info!("Card size is still {} bytes", volume_mgr.device().num_bytes().unwrap());

    let volume0 = volume_mgr.open_volume(VolumeIdx(0)).unwrap();

    log::info!("Volume 0: {:?}", volume0);




}
*/
