use std::error::Error;
use std::process::{Command, Stdio};
use std::io::{self, BufRead};

pub fn RunPm3() -> Result<(), Box<dyn Error>> {
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
                let parse_result = super::parser::ParseLine(&line);
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

    Ok(())
}
