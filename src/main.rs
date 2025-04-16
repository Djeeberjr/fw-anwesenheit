use pm3::run_pm3;

mod parser;
mod pm3;
mod id_store;
mod buzzer;

fn main() {
    run_pm3().unwrap();
}
