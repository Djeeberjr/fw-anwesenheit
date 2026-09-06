use alloc::{string::ToString, vec::Vec};
use core::str::from_utf8;
use embassy_time::Delay;
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_sdmmc::{
    SdCard, SdCardError, ShortFileName, TimeSource, Timestamp, VolumeIdx, VolumeManager,
};
use esp_hal::{Blocking, gpio::Output, spi::master::Spi};
use thiserror::Error;

use crate::store::{
    AttendanceDay, day::Day, mapping_loader::Name, persistence::Persistence, tally_id::TallyID,
};

pub struct DummyTimesource;

impl TimeSource for DummyTimesource {
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

pub type VolMgr = VolumeManager<
    SdCard<ExclusiveDevice<Spi<'static, Blocking>, Output<'static>, Delay>, Delay>,
    DummyTimesource,
>;

pub fn setup_sdcard(spi_bus: Spi<'static, Blocking>, cs_pin: Output<'static>) -> SDCardPersistence {
    let spi_device = ExclusiveDevice::new(spi_bus, cs_pin, Delay).unwrap();
    let sd_card = SdCard::new(spi_device, Delay);
    let vol_mgr = VolumeManager::new(sd_card, DummyTimesource);

    SDCardPersistence { vol_mgr }
}

pub struct SDCardPersistence {
    vol_mgr: VolMgr,
}

impl SDCardPersistence {
    const MAPPING_DIRNAME: &'static str = "MAPPINGS";

    fn generate_filename_for_day(day: Day) -> Result<ShortFileName, PersistenceError> {
        let basename = day.to_string();
        let mut filename: heapless::String<11> = heapless::String::new();
        filename
            .push_str(&basename)
            .map_err(|_| PersistenceError::DayFilename)?;
        filename
            .push_str(".JS")
            .map_err(|_| PersistenceError::DayFilename)?;

        ShortFileName::create_from_str(&filename).map_err(|_| PersistenceError::DayFilename)
    }

    fn generate_path_for_id(
        id: TallyID,
    ) -> Result<(ShortFileName, ShortFileName), PersistenceError> {
        let basename: heapless::String<12> = id.into();
        let (dir, file) = basename.split_at(6);

        let mut filename: heapless::String<11> = heapless::String::new();
        filename
            .push_str(file)
            .map_err(|_| PersistenceError::IDFilename)?;
        filename
            .push_str(".JS")
            .map_err(|_| PersistenceError::IDFilename)?;

        let mut dirname: heapless::String<11> = heapless::String::new();
        dirname
            .push_str(dir)
            .map_err(|_| PersistenceError::IDFilename)?;

        Ok((
            ShortFileName::create_from_str(&dirname).map_err(|_| PersistenceError::IDFilename)?,
            ShortFileName::create_from_str(&filename).map_err(|_| PersistenceError::IDFilename)?,
        ))
    }

    fn get_tallyid_from_path(
        dirname: &ShortFileName,
        filename: &ShortFileName,
    ) -> Result<TallyID, PersistenceError> {
        let mut id_str: heapless::String<12> = heapless::String::new();

        id_str
            .push_str(&dirname.to_string())
            .map_err(|_| PersistenceError::IDFilename)?;
        id_str
            .push_str(from_utf8(filename.base_name()).map_err(|_| PersistenceError::IDFilename)?)
            .map_err(|_| PersistenceError::IDFilename)?;

        let id: TallyID = id_str
            .try_into()
            .map_err(|_| PersistenceError::IDFilename)?;

        Ok(id)
    }
}

#[derive(Error, Debug)]
pub enum PersistenceError {
    #[error("Failed to interact with SD card")]
    SdCard(embedded_sdmmc::Error<SdCardError>),

    #[error("Failed to parse data")]
    Parseing(#[from] serde_json::Error),

    #[error("Failed to parse Day and Filename")]
    DayFilename,

    #[error("Failed to parse TallyID for file path")]
    IDFilename,

    #[error("Item not found")]
    NotFound,
}

impl From<embedded_sdmmc::Error<SdCardError>> for PersistenceError {
    fn from(err: embedded_sdmmc::Error<SdCardError>) -> Self {
        PersistenceError::SdCard(err)
    }
}

impl Persistence for SDCardPersistence {
    type Error = PersistenceError;

    async fn load_day(&mut self, day: Day) -> Result<AttendanceDay, Self::Error> {
        let mut vol_0 = self.vol_mgr.open_volume(VolumeIdx(0))?;
        let mut root_dir = vol_0.open_root_dir()?;

        let filename = Self::generate_filename_for_day(day)?;
        let file = root_dir.open_file_in_dir(filename, embedded_sdmmc::Mode::ReadOnly);

        if file.is_err() {
            return Err(PersistenceError::NotFound);
        }

        let mut open_file = file?;

        let mut read_buffer: [u8; 1024] = [0; 1024];
        let read = open_file.read(&mut read_buffer)?;
        open_file.close()?;

        let day: AttendanceDay = serde_json::from_slice(&read_buffer[..read])?;

        Ok(day)
    }

    async fn save_day(&mut self, day: Day, data: &AttendanceDay) -> Result<(), Self::Error> {
        let mut vol_0 = self.vol_mgr.open_volume(VolumeIdx(0))?;
        let mut root_dir = vol_0.open_root_dir()?;

        let filename = Self::generate_filename_for_day(day)?;

        let mut file =
            root_dir.open_file_in_dir(filename, embedded_sdmmc::Mode::ReadWriteCreateOrTruncate)?;

        file.write(&serde_json::to_vec(data)?)?;

        file.flush()?;
        file.close()?;

        Ok(())
    }

    async fn remove_day(&mut self, day: Day) -> Result<(), Self::Error> {
        let mut vol_0 = self.vol_mgr.open_volume(VolumeIdx(0))?;
        let mut root_dir = vol_0.open_root_dir()?;

        let filename = Self::generate_filename_for_day(day)?;

        root_dir.delete_file_in_dir(filename)?;

        Ok(())
    }

    async fn list_days(&mut self) -> Result<Vec<Day>, Self::Error> {
        let mut vol_0 = self.vol_mgr.open_volume(VolumeIdx(0))?;
        let mut root_dir = vol_0.open_root_dir()?;

        let mut days_dir = root_dir.open_dir(".")?;

        let mut days: Vec<Day> = Vec::new();
        days_dir.iterate_dir(|e| {
            let filename = e.name.clone();

            if let Ok(day) = filename.try_into() {
                days.push(day);
            }
        })?;

        Ok(days)
    }

    async fn load_mapping_for_id(
        &mut self,
        id: crate::store::tally_id::TallyID,
    ) -> Result<crate::store::mapping_loader::Name, Self::Error> {
        let mut vol_0 = self.vol_mgr.open_volume(VolumeIdx(0))?;
        let mut root_dir = vol_0.open_root_dir()?;
        let mut mapping_dir = root_dir.open_dir(Self::MAPPING_DIRNAME)?;

        let (dirname, filename) = Self::generate_path_for_id(id)?;

        let mut dir = mapping_dir.open_dir(dirname)?;
        let mut file = dir.open_file_in_dir(filename, embedded_sdmmc::Mode::ReadOnly)?;

        let mut read_buffer: [u8; 1024] = [0; 1024];
        let read_bytes = file.read(&mut read_buffer)?;
        file.close()?;

        let mapping: Name = serde_json::from_slice(&read_buffer[..read_bytes])?;

        Ok(mapping)
    }

    async fn save_mapping_for_id(
        &mut self,
        id: crate::store::tally_id::TallyID,
        name: crate::store::mapping_loader::Name,
    ) -> Result<(), Self::Error> {
        let mut vol_0 = self.vol_mgr.open_volume(VolumeIdx(0))?;
        let mut root_dir = vol_0.open_root_dir()?;
        let mut mapping_dir = root_dir.open_dir(Self::MAPPING_DIRNAME)?;

        let (dirname, filename) = Self::generate_path_for_id(id)?;

        let mut dir = if let Ok(dir) = mapping_dir.open_dir(&dirname) {
            dir
        } else {
            mapping_dir.make_dir_in_dir(&dirname)?;
            mapping_dir.open_dir(&dirname)?
        };

        let mut file =
            dir.open_file_in_dir(filename, embedded_sdmmc::Mode::ReadWriteCreateOrTruncate)?;

        file.write(&serde_json::to_vec(&name)?)?;

        Ok(())
    }

    async fn list_mappings(&mut self) -> Result<Vec<TallyID>, Self::Error> {
        let mut vol_0 = self.vol_mgr.open_volume(VolumeIdx(0))?;
        let mut root_dir = vol_0.open_root_dir()?;
        let mut mapping_dir = root_dir.open_dir(Self::MAPPING_DIRNAME)?;
        let mut ids: Vec<TallyID> = Vec::new();

        let mut dir_names = Vec::new();
        mapping_dir.iterate_dir(|entry| {
            if entry.attributes.is_directory()
                && entry.name.to_string() != "."
                && entry.name.to_string() != ".."
            {
                dir_names.push(entry.name.clone());
            }
        })?;

        for dirname in dir_names {
            if let Ok(mut subdir) = mapping_dir.open_dir(&dirname) {
                let mut file_names = Vec::new();
                subdir.iterate_dir(|file_entry| {
                    if !file_entry.attributes.is_directory() {
                        file_names.push(file_entry.name.clone());
                    }
                })?;

                for filename in file_names {
                    let id = Self::get_tallyid_from_path(&dirname, &filename)?;
                    ids.push(id);
                }
            }
        }

        Ok(ids)
    }
}
