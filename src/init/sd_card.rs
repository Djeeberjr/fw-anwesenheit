use core::str::from_utf8;

use alloc::{string::ToString, vec::Vec};
use embassy_time::Delay;
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_sdmmc::{SdCard, ShortFileName, TimeSource, Timestamp, VolumeIdx, VolumeManager};
use esp_hal::{Blocking, gpio::Output, spi::master::Spi};

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

    fn generate_filename_for_day(day: Day) -> ShortFileName {
        let basename = day.to_string();
        let mut filename: heapless::String<11> = heapless::String::new();
        filename.push_str(&basename).unwrap();
        filename.push_str(".JS").unwrap();

        ShortFileName::create_from_str(&filename).unwrap()
    }

    fn generate_path_for_id(id: TallyID) -> (ShortFileName, ShortFileName) {
        let basename: heapless::String<12> = id.into();
        let (dir, file) = basename.split_at(6);

        let mut filename: heapless::String<11> = heapless::String::new();
        filename.push_str(file).unwrap();
        filename.push_str(".JS").unwrap();

        let mut dirname: heapless::String<11> = heapless::String::new();
        dirname.push_str(dir).unwrap();

        (
            ShortFileName::create_from_str(&dirname).unwrap(),
            ShortFileName::create_from_str(&filename).unwrap(),
        )
    }

    fn get_tallyid_from_path(dirname: &ShortFileName, filename: &ShortFileName) -> Option<TallyID> {
        let mut id_str: heapless::String<12> = heapless::String::new();

        id_str.push_str(&dirname.to_string()).unwrap();
        id_str
            .push_str(from_utf8(filename.base_name()).unwrap())
            .unwrap();

        let id: TallyID = id_str.try_into().unwrap();

        Some(id)
    }
}

impl Persistence for SDCardPersistence {
    async fn load_day(&mut self, day: Day) -> Option<AttendanceDay> {
        let mut vol_0 = self.vol_mgr.open_volume(VolumeIdx(0)).unwrap();
        let mut root_dir = vol_0.open_root_dir().unwrap();

        let filename = Self::generate_filename_for_day(day);
        let file = root_dir.open_file_in_dir(filename, embedded_sdmmc::Mode::ReadOnly);

        if file.is_err() {
            return None;
        }

        let mut open_file = file.unwrap();

        let mut read_buffer: [u8; 1024] = [0; 1024];
        let read = open_file.read(&mut read_buffer).unwrap();
        open_file.close().unwrap();

        let day: AttendanceDay = serde_json::from_slice(&read_buffer[..read]).unwrap();

        Some(day)
    }

    async fn save_day(&mut self, day: Day, data: &AttendanceDay) {
        let mut vol_0 = self.vol_mgr.open_volume(VolumeIdx(0)).unwrap();
        let mut root_dir = vol_0.open_root_dir().unwrap();

        let filename = Self::generate_filename_for_day(day);

        let mut file = root_dir
            .open_file_in_dir(filename, embedded_sdmmc::Mode::ReadWriteCreateOrTruncate)
            .unwrap();

        file.write(&serde_json::to_vec(data).unwrap()).unwrap();

        file.flush().unwrap();
        file.close().unwrap();
    }

    async fn list_days(&mut self) -> Vec<Day> {
        let mut vol_0 = self.vol_mgr.open_volume(VolumeIdx(0)).unwrap();
        let mut root_dir = vol_0.open_root_dir().unwrap();

        let mut days_dir = root_dir.open_dir(".").unwrap();

        let mut days: Vec<Day> = Vec::new();
        days_dir
            .iterate_dir(|e| {
                let filename = e.name.clone();

                if let Ok(day) = filename.try_into() {
                    days.push(day);
                }
            })
            .unwrap();

        days
    }

    async fn load_mapping_for_id(
        &mut self,
        id: crate::store::tally_id::TallyID,
    ) -> Option<crate::store::mapping_loader::Name> {
        let mut vol_0 = self.vol_mgr.open_volume(VolumeIdx(0)).unwrap();
        let mut root_dir = vol_0.open_root_dir().unwrap();
        let mut mapping_dir = root_dir.open_dir(Self::MAPPING_DIRNAME).unwrap();

        let (dirname, filename) = Self::generate_path_for_id(id);

        let mut dir = mapping_dir.open_dir(dirname).unwrap();
        let mut file = dir
            .open_file_in_dir(filename, embedded_sdmmc::Mode::ReadOnly)
            .unwrap();

        let mut read_buffer: [u8; 1024] = [0; 1024];
        let read_bytes = file.read(&mut read_buffer).unwrap();
        file.close().unwrap();

        let mapping: Name = serde_json::from_slice(&read_buffer[..read_bytes]).unwrap();

        Some(mapping)
    }

    async fn save_mapping_for_id(
        &mut self,
        id: crate::store::tally_id::TallyID,
        name: crate::store::mapping_loader::Name,
    ) {
        let mut vol_0 = self.vol_mgr.open_volume(VolumeIdx(0)).unwrap();
        let mut root_dir = vol_0.open_root_dir().unwrap();
        let mut mapping_dir = root_dir.open_dir(Self::MAPPING_DIRNAME).unwrap();

        let (dirname, filename) = Self::generate_path_for_id(id);

        let mut dir = if let Ok(dir) = mapping_dir.open_dir(&dirname) {
            dir
        } else {
            mapping_dir.make_dir_in_dir(&dirname).unwrap();
            mapping_dir.open_dir(&dirname).unwrap()
        };

        let mut file = dir
            .open_file_in_dir(filename, embedded_sdmmc::Mode::ReadWriteCreateOrTruncate)
            .unwrap();

        file.write(&serde_json::to_vec(&name).unwrap()).unwrap();
    }

    async fn list_mappings(&mut self) -> Vec<TallyID> {
        let mut vol_0 = self.vol_mgr.open_volume(VolumeIdx(0)).unwrap();
        let mut root_dir = vol_0.open_root_dir().unwrap();
        let mut mapping_dir = root_dir.open_dir(Self::MAPPING_DIRNAME).unwrap();
        let mut ids: Vec<TallyID> = Vec::new();

        let mut dir_names = Vec::new();
        mapping_dir
            .iterate_dir(|entry| {
                if entry.attributes.is_directory()
                    && entry.name.to_string() != "."
                    && entry.name.to_string() != ".."
                {
                    dir_names.push(entry.name.clone());
                }
            })
            .unwrap();

        for dirname in dir_names {
            if let Ok(mut subdir) = mapping_dir.open_dir(&dirname) {
                let mut file_names = Vec::new();
                subdir
                    .iterate_dir(|file_entry| {
                        if !file_entry.attributes.is_directory() {
                            file_names.push(file_entry.name.clone());
                        }
                    })
                    .unwrap();

                for filename in file_names {
                    let id = Self::get_tallyid_from_path(&dirname, &filename);
                    ids.push(id.unwrap());
                }
            }
        }

        ids
    }
}
