use std::error::Error;
use std::io::{self, BufRead};
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;
use std::{thread, time};

pub fn run_pm3(sender: Sender<String>) -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::new("stdbuf")
        .arg("-oL")
        .arg("pm3")
        .arg("-c")
        .arg("lf hitag reader -@")
        .stdout(Stdio::piped())
        .spawn()?;

    let stdout = cmd.stdout.take().ok_or("Failed to get stdout")?;
    let reader = io::BufReader::new(stdout);

    for line_result in reader.lines() {
        match line_result {
            Ok(line) => {
                let parse_result = super::parser::parse_line(&line);
                if let Some(uid) = parse_result {
                    match sender.send(uid) {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("Failed to send to channel: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("{}", e);
            }
        }
    }

    let status = cmd.wait().expect("Failed to wait on child");

    if status.success() {
        Ok(())
    } else {
        Err("pm3 had non zero exit code".into())
    }
}

pub fn pm3_mock(sender: Sender<String>) -> Result<(), Box<dyn Error>> {
    #![allow(while_true)]
    while true {
        match sender.send("F1409618".to_owned()) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("Failed to send to channel: {}", e);
            }
        }

        thread::sleep(time::Duration::from_secs(2));
    }

    Ok(())
}
