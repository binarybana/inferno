import time
import struct
import asyncio

from typing import List, Tuple, Iterable

from machine import Pin, PWM
import select
import sys


led = PWM(Pin("LED"))
ember = PWM(Pin("GP16"))
relay = Pin("GP0", Pin.OUT)
relay.value(0)

DATA_CAP = 10000


ember.freq(1000)
led.freq(1000)


def load_brightness_data(fname: str) -> Tuple[float]:
    with open(fname, "rb") as f:
        # Read the number of values
        num_values = struct.unpack("!I", f.read(4))[0]
        num_values = min(num_values, DATA_CAP)
        # Read each brightness value
        return struct.unpack("!" + str(num_values) + "f", f.read(4 * num_values))


def scale_brightness_data(
    values: Iterable[float], max_brightness: float, dynamic_range: float
) -> List[int]:
    """
    max_brightness: float between 0 and 1 indicating how bright the user wants the bed to ever reach
    dynamic_range: float between 0 and 1 indicating how much of the remaining portion of usable output the user wants to use
    """
    min_val = min(values)
    max_val = max(values)
    max_desired_val = max_brightness * (2**16 - 1)
    min_desired_val = max_desired_val * dynamic_range
    observed_range = max_val - min_val
    desired_range = max_desired_val - min_desired_val

    scale_factor = desired_range / observed_range
    shift_factor = (max_desired_val * (1 - dynamic_range)) - (min_val * scale_factor)
    # print(f"{min_val=} {max_val=} {scale_factor=} {shift_factor=}")
    return [int(shift_factor + scale_factor * x) for x in values]


async def update_pwm():
    brightess_lut = scale_brightness_data(
        load_brightness_data("packed_brightness.bin"),
        max_brightness=1.0,
        dynamic_range=0.5,
    )
    time_index = 0
    sleep_time = 1 / 60.0
    while True:
        # if time_index % 60 == 0:
        #     print(".", end="")
        brightness = brightess_lut[time_index]
        led.duty_u16(brightness)
        ember.duty_u16(brightness)
        await asyncio.sleep(sleep_time)
        time_index += 1
        time_index %= len(brightess_lut)


# Set up the poll object
poll_obj = select.poll()
poll_obj.register(sys.stdin, select.POLLIN)


async def control_loop():
    while True:
        poll_results = poll_obj.poll(1)
        if poll_results:
            data = sys.stdin.readline().strip()
            if data == "on":
                relay.value(1)
            if data == "off":
                relay.value(0)
            sys.stdout.write("received data: " + data + "\r")
        else:
            await asyncio.sleep(0.1)


async def main():
    task1 = asyncio.create_task(control_loop())
    task2 = asyncio.create_task(update_pwm())
    await task1
    await task2


if __name__ == "__main__":
    asyncio.run(main())
