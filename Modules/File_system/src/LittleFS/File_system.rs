use core::{mem::MaybeUninit, pin::Pin};
use std::{collections::BTreeMap, ffi::CString, sync::RwLock};

use crate::{
    File_system_traits, Local_file_identifier_type, Mode_type, Path_type, Size_type,
    Statistics_type, Unique_file_identifier_type,
};

use super::{littlefs, Configuration_type, Convert_result, Error_type, File_type, Result_type};

struct Inner_type<const Cache_size: usize> {
    Configuration: littlefs::lfs_config,
    Read_buffer: [u8; Cache_size],
    Write_buffer: [u8; Cache_size],
    Lookahead_buffer: [u8; Cache_size],
}

struct Inner_2_type {
    File_system: littlefs::lfs_t,
    Open_files: BTreeMap<Local_file_identifier_type, File_type>,
}

pub struct File_system_type<const Cache_size: usize> {
    Inner: Pin<Box<Inner_type<Cache_size>>>,
    Inner_2: RwLock<Inner_2_type>,
}

impl<const Cache_size: usize> Drop for File_system_type<Cache_size> {
    fn drop(&mut self) {
        let mut Inner = self.Inner_2.write().unwrap();

        let Keys = Inner.Open_files.keys().cloned().collect::<Vec<_>>();

        for Key in Keys {
            if let Some(File) = Inner.Open_files.remove(&Key) {
                let _ = File.Close(&mut Inner.File_system);
            }
        }

        unsafe {
            littlefs::lfs_unmount(&mut Inner.File_system as *mut _);
        }
    }
}

impl<const Cache_size: usize> File_system_type<Cache_size> {
    pub fn New(
        Device_file: Unique_file_identifier_type,
        Configuration: Configuration_type,
    ) -> Result_type<Self> {
        let mut Inner = Box::pin(Inner_type {
            Configuration: unsafe { MaybeUninit::uninit().assume_init() },
            Read_buffer: [0; Cache_size],
            Write_buffer: [0; Cache_size],
            Lookahead_buffer: [0; Cache_size],
        });

        let mut Inner_reference = Inner.as_mut();

        let Configuration: littlefs::lfs_config = Configuration
            .Set_buffers(
                Inner_reference.Read_buffer.as_mut_ptr(),
                Inner_reference.Write_buffer.as_mut_ptr(),
                Inner_reference.Lookahead_buffer.as_mut_ptr(),
            )
            .Set_context(Device_file)
            .try_into()
            .map_err(|_| Error_type::Invalid_parameter)?;

        Inner_reference.Configuration = Configuration;

        let Configuration_pointer = &Inner_reference.Configuration as *const _;

        let mut File_system = unsafe { MaybeUninit::<littlefs::lfs_t>::uninit().assume_init() };

        Convert_result(unsafe {
            littlefs::lfs_mount(&mut File_system as *mut _, Configuration_pointer)
        })?;

        let Inner_2 = Inner_2_type {
            File_system,
            Open_files: BTreeMap::new(),
        };

        Ok(Self {
            Inner,
            Inner_2: RwLock::new(Inner_2),
        })
    }

    pub fn Format(
        Device_file: Unique_file_identifier_type,
        Configuration: Configuration_type,
    ) -> Result_type<()> {
        let mut Inner = Box::pin(Inner_type {
            Configuration: unsafe { MaybeUninit::uninit().assume_init() },
            Read_buffer: [0; Cache_size],
            Write_buffer: [0; Cache_size],
            Lookahead_buffer: [0; Cache_size],
        });

        let mut Inner_reference = Inner.as_mut();

        let Configuration: littlefs::lfs_config = Configuration
            .Set_buffers(
                Inner_reference.Read_buffer.as_mut_ptr(),
                Inner_reference.Write_buffer.as_mut_ptr(),
                Inner_reference.Lookahead_buffer.as_mut_ptr(),
            )
            .Set_context(Device_file)
            .try_into()
            .map_err(|_| Error_type::Invalid_parameter)?;

        Inner_reference.Configuration = Configuration;

        let Configuration_pointer = &Inner_reference.Configuration as *const _;

        let mut File_system = unsafe { MaybeUninit::<littlefs::lfs_t>::uninit().assume_init() };

        Convert_result(unsafe {
            littlefs::lfs_format(&mut File_system as *mut _, Configuration_pointer)
        })?;

        Ok(())
    }

    fn Get_new_file_identifier<T>(
        Task_identifier: Task::Task_identifier_type,
        Open_files: &BTreeMap<Local_file_identifier_type, T>,
    ) -> Result_type<Local_file_identifier_type> {
        let Iterator = Local_file_identifier_type::Get_minimum(Task_identifier);

        for Identifier in Iterator {
            if !Open_files.contains_key(&Identifier) {
                return Ok(Identifier);
            }
        }

        Err(Error_type::Too_many_open_files)
    }

    fn Borrow_mutable_inner_2_splited<'a>(
        Inner_2: &'a mut Inner_2_type,
    ) -> (
        &'a mut littlefs::lfs_t,
        &'a mut BTreeMap<Local_file_identifier_type, File_type>,
    ) {
        (&mut Inner_2.File_system, &mut Inner_2.Open_files)
    }
}

unsafe impl<const Buffer_size: usize> Send for File_system_type<Buffer_size> {}

unsafe impl<const Buffer_size: usize> Sync for File_system_type<Buffer_size> {}

impl<const Buffer_size: usize> File_system_traits for File_system_type<Buffer_size> {
    fn Open(
        &self,
        Task: Task::Task_identifier_type,
        Path: &dyn AsRef<Path_type>,
        Flags: crate::Flags_type,
    ) -> crate::Result_type<Local_file_identifier_type> {
        let mut Inner = self.Inner_2.write()?;

        let File = File_type::Open(&mut Inner.File_system, Task, Path, Flags)?;

        let File_identifier = Self::Get_new_file_identifier(Task, &Inner.Open_files)?;

        if Inner.Open_files.insert(File_identifier, File).is_some() {
            return Err(Error_type::Internal_error.into());
        }

        Ok(File_identifier)
    }

    fn Close(&self, File: Local_file_identifier_type) -> crate::Result_type<()> {
        let mut Inner = self.Inner_2.write()?;

        let File = Inner
            .Open_files
            .remove(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        File.Close(&mut Inner.File_system)?;

        Ok(())
    }

    fn Close_all(&self, Task: Task::Task_identifier_type) -> crate::Result_type<()> {
        let mut Inner = self.Inner_2.write()?;

        // Get all the keys of the open files that belong to the task
        let Keys = Inner
            .Open_files
            .keys()
            .filter(|Key| Key.Split().0 == Task)
            .cloned()
            .collect::<Vec<_>>();

        // Close all the files
        for Key in Keys {
            if let Some(File) = Inner.Open_files.remove(&Key) {
                File.Close(&mut Inner.File_system)?;
            }
        }

        Ok(())
    }

    fn Duplicate_file_identifier(
        &self,
        File: Local_file_identifier_type,
    ) -> crate::Result_type<Local_file_identifier_type> {
        let (Task, _) = File.Split();

        let mut Inner = self.Inner_2.write()?;

        let File = Inner
            .Open_files
            .get(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        let File = File.clone();

        let File_identifier = Self::Get_new_file_identifier(Task, &Inner.Open_files)?;

        if Inner.Open_files.insert(File_identifier, File).is_some() {
            return Err(Error_type::Internal_error.into());
        }

        Ok(File_identifier)
    }

    fn Transfert_file_identifier(
        &self,
        New_task: Task::Task_identifier_type,
        File: Local_file_identifier_type,
    ) -> crate::Result_type<Local_file_identifier_type> {
        let mut Inner = self.Inner_2.write()?;

        let File = Inner
            .Open_files
            .remove(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        let File_identifier = Self::Get_new_file_identifier(New_task, &Inner.Open_files)?;

        if Inner.Open_files.insert(File_identifier, File).is_some() {
            return Err(Error_type::Internal_error.into());
        }

        Ok(File_identifier)
    }

    fn Delete(&self, Path: &dyn AsRef<Path_type>) -> crate::Result_type<()> {
        let Path =
            CString::new(Path.as_ref().As_str()).map_err(|_| Error_type::Invalid_parameter)?;

        let mut Inner = self.Inner_2.write()?;

        Convert_result(unsafe {
            littlefs::lfs_remove(&mut Inner.File_system as *mut _, Path.as_ptr())
        })?;

        Ok(())
    }

    fn Read(
        &self,
        File: Local_file_identifier_type,
        Buffer: &mut [u8],
    ) -> crate::Result_type<Size_type> {
        let mut Inner = self.Inner_2.write()?;

        let (File_system, Open_files) = Self::Borrow_mutable_inner_2_splited(&mut Inner);

        let File = Open_files
            .get_mut(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        Ok(File.Read(File_system, Buffer)?)
    }

    fn Write(
        &self,
        File: Local_file_identifier_type,
        Buffer: &[u8],
    ) -> crate::Result_type<Size_type> {
        let mut Inner = self.Inner_2.write()?;

        let (File_system, Open_files) = Self::Borrow_mutable_inner_2_splited(&mut Inner);

        let File = Open_files
            .get_mut(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        Ok(File.Write(File_system, Buffer)?)
    }

    fn Move(
        &self,
        Source: &dyn AsRef<Path_type>,
        Destination: &dyn AsRef<Path_type>,
    ) -> crate::Result_type<()> {
        let Source =
            CString::new(Source.as_ref().As_str()).map_err(|_| Error_type::Invalid_parameter)?;

        let Destination = CString::new(Destination.as_ref().As_str())
            .map_err(|_| Error_type::Invalid_parameter)?;

        let mut Inner = self.Inner_2.write()?;

        Convert_result(unsafe {
            littlefs::lfs_rename(
                &mut Inner.File_system as *mut _,
                Source.as_ptr(),
                Destination.as_ptr(),
            )
        })?;

        Ok(())
    }

    fn Set_position(
        &self,
        File: Local_file_identifier_type,
        Position: &crate::Position_type,
    ) -> crate::Result_type<Size_type> {
        let mut Inner = self.Inner_2.write()?;

        let (File_system, Open_files) = Self::Borrow_mutable_inner_2_splited(&mut Inner);

        let File = Open_files
            .get_mut(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        Ok(File.Set_position(File_system, Position)?)
    }

    fn Flush(&self, File: Local_file_identifier_type) -> crate::Result_type<()> {
        let mut Inner = self.Inner_2.write()?;

        let (File_system, Open_files) = Self::Borrow_mutable_inner_2_splited(&mut Inner);

        let File = Open_files
            .get_mut(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        Ok(File.Flush(File_system)?)
    }

    fn Get_statistics(
        &self,
        File: Local_file_identifier_type,
    ) -> crate::Result_type<Statistics_type> {
        let mut Inner = self.Inner_2.write()?;

        let (File_system, Open_files) = Self::Borrow_mutable_inner_2_splited(&mut Inner);

        let File = Open_files
            .get_mut(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        Ok(File.Get_statistics(File_system)?)
    }

    fn Get_mode(&self, File: Local_file_identifier_type) -> crate::Result_type<Mode_type> {
        Ok(self
            .Inner_2
            .read()?
            .Open_files
            .get(&File)
            .ok_or(Error_type::Invalid_identifier)?
            .Get_mode())
    }
}