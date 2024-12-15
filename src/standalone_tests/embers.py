import time
from machine import Pin, PWM, Timer

led = PWM(Pin("LED"))
ember = PWM(Pin("GP16"))
tim = Timer()
relay = Pin("GP0", Pin.OUT)

ember.freq(1000)
led.freq(1000)


def tick(timer):
    global relay
    relay.toggle()


tim.init(freq=0.2, mode=Timer.PERIODIC, callback=tick)

duty = 0
direction = 1
while True:
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
