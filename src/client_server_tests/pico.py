import time
from machine import Pin, PWM, Timer
import select
import sys
import time


led = PWM(Pin("LED"))
ember = PWM(Pin("GP16"))
tim = Timer()
relay = Pin("GP0", Pin.OUT)
duty = 0
direction = 1

ember.freq(1000)
led.freq(1000)


def tick(timer):
    global relay
    relay.toggle()


def update_pwm():
    global duty, direction
    duty += direction
    if duty > 255:
        duty = 255
        direction = -1
    elif duty < 0:
        duty = 0
        direction = 1
    led.duty_u16(duty * duty)
    ember.duty_u16(duty * duty)
    time.sleep(0.001)


# Set up the poll object
poll_obj = select.poll()
poll_obj.register(sys.stdin, select.POLLIN)

# Start tick timer
# tim.init(freq=0.2, mode=Timer.PERIODIC, callback=tick)

# Loop indefinitely
while True:
    # Wait for input on stdin
    poll_results = poll_obj.poll(
        1
    )  # the '1' is how long it will wait for message before looping again (in microseconds)
    if poll_results:
        # Read the data from stdin (read data coming from PC)
        data = sys.stdin.readline().strip()
        if data == "on":
            relay.value(1)
        if data == "off":
            relay.value(0)
        # Write the data to the input file
        sys.stdout.write("received data: " + data + "\r")
    else:
        # do something if no message received (like feed a watchdog timer)
        update_pwm()
        continue
