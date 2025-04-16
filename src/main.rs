use pm3::{pm3_mock, run_pm3};
use std::{sync::mpsc::channel, thread};

mod id_store;
mod parser;
mod pm3;

fn main() {
    let (sender, receiver) = channel();
    thread::spawn(move || {
        // run_pm3(sender);
        pm3_mock(sender);
    });

    while true {
        println!("{}", receiver.recv().unwrap());
    }
}
