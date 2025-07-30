use std::env;
use std::path::Path;
use std::fs::File;
use std::io::Write;

fn main() {
    linker_be_nice();
    // make sure linkall.x is the last linker script (otherwise might cause problems with flip-link)
    println!("cargo:rustc-link-arg=-Tlinkall.x");
    save_build_time();
}

fn save_build_time() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("build_time.rs");
    let system_time = std::time::SystemTime::now();
    let unix_time = system_time
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    println!("cargo:rustc-env=BUILD_TIME={}", unix_time);
    let content = format!(
        "/// compile time as UNIX-Timestamp (seconds since 1970-01-01)
        pub const BUILD_UNIX_TIME: u64 = {};",
        unix_time
    );
    let mut f = File::create(dest_path).unwrap();
    f.write_all(content.as_bytes()).unwrap();
}

fn linker_be_nice() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let kind = &args[1];
        let what = &args[2];

        match kind.as_str() {
            "undefined-symbol" => match what.as_str() {
                "_defmt_timestamp" => {
                    eprintln!();
                    eprintln!(
                        "💡 `defmt` not found - make sure `defmt.x` is added as a linker script and you have included `use defmt_rtt as _;`"
                    );
                    eprintln!();
                }
                "_stack_start" => {
                    eprintln!();
                    eprintln!("💡 Is the linker script `linkall.x` missing?");
                    eprintln!();
                }
                _ => (),
            },
            // we don't have anything helpful for "missing-lib" yet
            _ => {
                std::process::exit(1);
            }
        }

        std::process::exit(0);
    }

    println!(
        "cargo:rustc-link-arg=--error-handling-script={}",
        std::env::current_exe().unwrap().display()
    );
}
