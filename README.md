# Inferno

Controlling my digital fireplace

## Installation

Place the `.desktop` file in the `$HOME/.config/autostart/fireplace.desktop` location.

## Controlling the microcontroller

Launch the standalone embers program.
```
uvx mpremote run src/standalone_tests/embers.py
```
## Sampling video data for realistic ember brightness time samplings

`ffplay -vf "drawbox=x=1700:y=1600:w=50:h=50:color=red:t=2" ~/tmp/video2.webm`

`ffmpeg -i ~/tmp/video2.webm -vf "crop=w=50:h=50:x=1700:y=1600,format=gray" -f rawvideo - | uv run python b2.py`

`uv run python analysis.py`

## Runtime stats from the microcontroller
`$ uvx mpremote repl`

```python
import gc
import os
import machine

s = os.statvfs('/')
print(f"Free storage: {s[0]*s[3]/1024} KB")
print(f"Memory: {gc.mem_alloc()} of {gc.mem_free()} bytes used.")
print(f"CPU Freq: {machine.freq()/1000000}Mhz")
```

## TODO
 * Figure out why MPV socket is not working right
   * Use it for more control
 * Have an automatic shutoff when the heater is on for more than 4 hours
 * Automatically 
