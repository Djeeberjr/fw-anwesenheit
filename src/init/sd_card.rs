use alloc::vec::Vec;
use embassy_time::Delay;
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_sdmmc::{SdCard, TimeSource, Timestamp, VolumeIdx, VolumeManager};
use esp_hal::{Blocking, gpio::Output, spi::master::Spi};

use crate::store::{AttendanceDay, Date, persistence::Persistence};

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

impl Persistence for SDCardPersistence {
    async fn load_day(&mut self, day: crate::store::Date) -> Option<AttendanceDay> {
        let mut vol_0 = self.vol_mgr.open_volume(VolumeIdx(0)).unwrap();
        let mut root_dir = vol_0.open_root_dir().unwrap();
        let mut file = root_dir
            .open_file_in_dir("days/TODO", embedded_sdmmc::Mode::ReadOnly)
            .unwrap();

        let mut read_buffer: [u8; 1024] = [0; 1024];
        let read = file.read(&mut read_buffer).unwrap();
        file.close().unwrap();

        let day: AttendanceDay = serde_json::from_slice(&read_buffer).unwrap();

        Some(day)
    }

    async fn save_day(&mut self, day: Date, data: &AttendanceDay) {
        todo!()
    }

    async fn load_mapping(&mut self) -> Option<crate::store::IDMapping> {
        todo!()
    }

    async fn save_mapping(&mut self, data: &crate::store::IDMapping) {
        todo!()
    }

    async fn list_days(&mut self) -> Vec<Date> {
        let mut vol_0 = self.vol_mgr.open_volume(VolumeIdx(0)).unwrap();
        let mut root_dir = vol_0.open_root_dir().unwrap();
        let mut days_dir = root_dir.open_dir("days").unwrap();

        let mut days = Vec::new();
        days_dir
            .iterate_dir(|e| {
                days.push(1);
            })
            .unwrap();

        days
    }
}
