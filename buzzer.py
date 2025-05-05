import RPi.GPIO as GPIO
import time

BUZZER_PIN = 26

def beep(frequency, duration):
    pwm = GPIO.PWM(BUZZER_PIN, frequency)
    pwm.start(50)  # 50 % duty cycle
    time.sleep(duration)
    pwm.stop()

def main():
    GPIO.setmode(GPIO.BCM)
    GPIO.setup(BUZZER_PIN, GPIO.OUT)

    try:
        beep(523, 0.3)  # C5
        beep(659, 0.3)  # E5
        beep(784, 0.3)  # G5
    finally:
        GPIO.cleanup()

if __name__ == "__main__":
    main()