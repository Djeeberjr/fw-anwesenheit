use embassy_time::{Duration, Timer};
use log::warn;
use picoserve::response;

use crate::TallySubscriber;

pub struct IDEvents(pub TallySubscriber);

impl response::sse::EventSource for IDEvents {
    async fn write_events<W: picoserve::io::Write>(
        mut self,
        mut writer: response::sse::EventWriter<W>,
    ) -> Result<(), W::Error> {
        loop {
            let timeout = Timer::after(Duration::from_secs(15));
            let sel = embassy_futures::select::select(self.0.next_message(), timeout);

            match sel.await {
                embassy_futures::select::Either::First(msg) => match msg {
                    embassy_sync::pubsub::WaitResult::Message(id) => {
                        let id_str: heapless::String<12> = id.into();
                        writer.write_event("msg", id_str.as_str()).await?
                    }
                    embassy_sync::pubsub::WaitResult::Lagged(_) => {
                        warn!("SSE subscriber got lagged");
                    }
                },
                embassy_futures::select::Either::Second(_) => writer.write_keepalive().await?,
            }
        }
    }
}
