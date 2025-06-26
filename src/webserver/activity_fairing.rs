use std::time::Duration;

use log::error;
use rocket::{
    Data, Request,
    fairing::{Fairing, Info, Kind},
};
use tokio::{sync::mpsc, time::timeout};

pub struct ActivityNotifier {
    pub sender: mpsc::Sender<()>,
}

#[rocket::async_trait]
impl Fairing for ActivityNotifier {
    fn info(&self) -> Info {
        Info {
            name: "Keeps track of time since the last request",
            kind: Kind::Request | Kind::Response,
        }
    }

    async fn on_request(&self, _: &mut Request<'_>, _: &mut Data<'_>) {
        error!("on_request");
        let _ = self.sender.try_send(());
    }
}

pub fn spawn_idle_watcher<F>(idle_duration: Duration, mut on_idle: F) -> mpsc::Sender<()>
where
    F: FnMut() + Send + 'static,
{
    let (tx, mut rx) = mpsc::channel::<()>(100);

    tokio::spawn(async move {
        loop {
            let idle = timeout(idle_duration, rx.recv()).await;
            if idle.is_err() {
                // No activity received in the duration
                on_idle();
            }
        }
    });

    tx
}
