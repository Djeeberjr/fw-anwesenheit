use alloc::{format, vec::Vec};
use embassy_time::Delay;
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_sdmmc::{
    Directory, SdCard, ShortFileName, TimeSource, Timestamp, VolumeIdx, VolumeManager,
};
use esp_hal::{Blocking, gpio::Output, spi::master::Spi};

use crate::store::{AttendanceDay, day::Day, persistence::Persistence};

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
    fn generate_filename(day: Day) -> ShortFileName {
        let basename = day.to_string();
        let mut filename: heapless::String<11> = heapless::String::new();
        filename.push_str(&basename).unwrap();
        filename.push_str(".js").unwrap();

        ShortFileName::create_from_str(&filename).unwrap()
    }
}

impl Persistence for SDCardPersistence {
    async fn load_day(&mut self, day: Day) -> Option<AttendanceDay> {
        let mut vol_0 = self.vol_mgr.open_volume(VolumeIdx(0)).unwrap();
        let mut root_dir = vol_0.open_root_dir().unwrap();

        let filename = Self::generate_filename(day);
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

        let filename = Self::generate_filename(day);

        let mut file = root_dir
            .open_file_in_dir(filename, embedded_sdmmc::Mode::ReadWriteCreateOrTruncate)
            .unwrap();
        file.write(&serde_json::to_vec(data).unwrap()).unwrap();

        file.flush().unwrap();
        file.close().unwrap();
    }

    async fn load_mapping(&mut self) -> Option<crate::store::IDMapping> {
        todo!()
    }

    async fn save_mapping(&mut self, data: &crate::store::IDMapping) {
        todo!()
    }

    async fn list_days(&mut self) -> Vec<Day> {
        let mut vol_0 = self.vol_mgr.open_volume(VolumeIdx(0)).unwrap();
        let mut root_dir = vol_0.open_root_dir().unwrap();
        let mut days_dir = root_dir.open_dir("days").unwrap();

        let mut days: Vec<Day> = Vec::new();
        days_dir
            .iterate_dir(|e| {
                let filename = e.name.clone();
                let day: Day = filename.try_into().unwrap();
                days.push(day);
            })
            .unwrap();

        days
    }
}
