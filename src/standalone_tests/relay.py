from machine import Pin, Timer

led = Pin("LED", Pin.OUT)
relay = Pin("GP0", Pin.OUT)
tim = Timer()


def tick(timer):
    global led, relay
    led.toggle()
    relay.toggle()


tim.init(freq=0.5, mode=Timer.PERIODIC, callback=tick)
