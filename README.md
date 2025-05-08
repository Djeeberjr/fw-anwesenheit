# fw-anwesenheit

# Setup 

In order to use the LED we need to enable the SPI interface on the Rpi. 
You can enable it by running `sudo raspi-config`, or by manually adding `dtparam=spi=on` to `/boot/firmware/config.txt`.
Enable PWM ->  add dtoverlay=pwm to /boot/config.txt

# Config 

Environment variables:

- `PM3_BIN`: Path to the pm3 binary. Seach in path if not set. Can also be set to the `pm3_mock.sh` for testing.
- `LOG_LEVEL`: Can be set to either "debug","warn","error","trace" or "info". Defaults to "warn" in production.
- `HTTP_PORT`: What port to listen on. Defaults to 80.
