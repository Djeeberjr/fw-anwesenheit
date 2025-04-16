<<<<<<< HEAD
use std::error::Error;
use std::process::{Command, Stdio};
use std::io::{self, BufRead};

pub fn run_pm3() -> Result<(), Box<dyn Error>> {
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
                    println!("UID: {}",uid);
                }
            }
            Err(e) => {
                eprintln!("{}",e);
            }
        }
    }

    let status = cmd.wait().expect("Failed to wait on child");

    if status.success() {
        Ok(())
    }else {
        Err("pm3 had non zero exit code".into())
    }
}
=======
use std::error::Error;
use std::io::{self, BufRead};
use std::process::{Command, Stdio};
use tokio::time::{Duration, sleep};

use tokio::sync::mpsc;

pub async fn run_pm3(tx: mpsc::Sender<String>) -> Result<(), Box<dyn Error>> {
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
                    match tx.send(uid).await {
                        Ok(()) => {}
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

pub async fn pm3_mock(tx: mpsc::Sender<String>) -> Result<(), Box<dyn Error>> {
    #![allow(while_true)]
    while true {
        match tx.send("F1409618".to_owned()).await {
            Ok(()) => {}
            Err(e) => {
                eprintln!("Failed to send to channel: {}", e);
            }
        }

        sleep(Duration::from_millis(1000)).await;
    }

    Ok(())
}
>>>>>>> eb39b09632efb1568079352e3d639edc79df65fd
