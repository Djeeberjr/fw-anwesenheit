# fw-anwesenheit

# Setup 

In order to use the LED we need to enable the SPI interface on the Rpi. 
You can enable it by running `sudo raspi-config`, or by manually adding `dtparam=spi=on` to `/boot/firmware/config.txt`.
Enable PWM ->  add dtoverlay=pwm to /boot/config.txt
I²C fpr RTC `sudo raspi-config` -> interface -> enable I²C

# Config 

Flags:

`--error` or `-e`: Enters error state. The LED turns red and the hotspot is activated. This state gets called from systemd if the service is in a failure state.

Environment variables:

- `PM3_BIN`: Path to the pm3 binary. Seach in path if not set. Can also be set to the `pm3_mock.sh` for testing.
- `LOG_LEVEL`: Can be set to either "debug","warn","error","trace" or "info". Defaults to "warn" in production.
- `HTTP_PORT`: What port to listen on. Defaults to 80.
- `HOTSPOT_IDS`: A semicolon seperated list of ids to activate the hotspot with e.g. `578B5DF2;c1532b57`.
- `HOTSPOT_SSID`: Set the hotspot ssid. Defaults to "fwa".
- `HOTSPOT_PW`: Set the hotspot password. Default to "a9LG2kUVrsRRVUo1". Recommended to change.

Systemd:

The service is run as a systemd service. There are two service `fwa.service` and `fwa-fail.service`. They read their config 
from a env file located at `/etc/fwa.env`. See example [env file](service/fwa.env).
