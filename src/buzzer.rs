pub mod buzzer {
    use rppal::gpio::Gpio;
    use std::{thread, time};    


    /// Emits a sound on a passive buzzer.
    pub fn modulated_tone(pin_num: u8, carrier_hz: u32, sound_hz: u32, duration_ms: u64) {
        let gpio = Gpio::new().expect("GPIO konnte nicht initialisiert werden");
        let mut pin = gpio.get(pin_num).expect("Pin konnte nicht geöffnet werden").into_output();

        let carrier_period = time::Duration::from_micros((1_000_000.0 / carrier_hz as f64 / 2.0) as u64);
        let mod_period = 1_000.0 / sound_hz as f64; // in ms
        let total_cycles = duration_ms as f64 / mod_period;

        for _ in 0..total_cycles as u64 {
            // Modulation on: Carrier on for mod_period / 2
            let cycles_on = (carrier_hz as f64 * (mod_period / 2.0) / 1000.0) as u64;
            for _ in 0..cycles_on {
                pin.set_high();
                thread::sleep(carrier_period);
                pin.set_low();
                thread::sleep(carrier_period);
            }

            // Modulation off: Carrier on for mod_period / 2
            let pause = time::Duration::from_millis((mod_period / 2.0) as u64);
            thread::sleep(pause);
        }
    }

    pub fn beep_ack() {
        // GPIO 17, carrier  = 2300 Hz, sound = 440 Hz, Dauer = 1 sec
        modulated_tone(4, 2300, 500, 500);
        modulated_tone(4, 2300, 700, 500);
    }

    pub fn beep_nak() {
        // GPIO 17, carrier  = 2300 Hz, sound = 440 Hz, duration = 1 sec
        modulated_tone(4, 2300, 700, 500);
        modulated_tone(4, 2300, 500, 500);
    }

    pub fn beep_unnkown(){
        modulated_tone(4, 2300, 500, 500);
        modulated_tone(4, 2300, 500, 500);
        modulated_tone(4, 2300, 500, 500);
    }
}

