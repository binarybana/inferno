# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Inferno is a digital fireplace control system that integrates hardware control, video analysis, and headless operation. The project controls a physical fireplace with ember effects and heater functionality through microcontroller communication and keyboard input monitoring.

## Architecture

The project consists of three main components:

1. **Headless Control System** (`headless/`): Rust application that monitors keyboard input and controls the heater via serial communication. Uses evdev for input monitoring and serialport for microcontroller communication.

2. **Video Analysis Pipeline** (`scripts/`): Python tools for analyzing fireplace video content to extract realistic ember brightness patterns. Processes video data through FFmpeg and generates time-series data for ember simulation.

3. **Microcontroller Code** (`src/`): MicroPython code for Raspberry Pi Pico that controls ember LED patterns and relay switching based on serial commands or standalone operation.

## Common Commands

### Rust (Headless Controller)
```bash
cd headless
cargo build --release
cargo run -- --mpv-socket /tmp/mpvsocket
```

### Python (Video Analysis)
```bash
cd scripts
uv run python analysis.py          # Analyze brightness data
uv run python b2.py               # Process video frames (used with ffmpeg)
```

### Microcontroller Deployment
```bash
uvx mpremote run src/standalone_tests/embers.py    # Run ember simulation
uvx mpremote repl                                   # Access REPL for debugging
```

### Video Processing Pipeline
```bash
# Extract ember region and process brightness
ffmpeg -i ~/tmp/video2.webv -vf "crop=w=50:h=50:x=1700:y=1600,format=gray" -f rawvideo - | uv run python b2.py
uv run python analysis.py
```

## Hardware Integration

The system controls physical hardware through:
- Serial communication with Raspberry Pi Pico (`/dev/ttyACM0` at 115200 baud)
- Keyboard input monitoring via evdev (`/dev/input/event3`)
- PWM control of ember LEDs and relay switching for heater
- MPV integration for visual feedback via Unix socket (`/tmp/mpvsocket`)

## Key Files

- `headless/src/main.rs`: Main control loop for keyboard monitoring and heater control
- `src/standalone_tests/embers.py`: MicroPython ember simulation with PWM control
- `scripts/analysis.py`: Video brightness analysis and data visualization
- `scripts/b2.py`: Real-time video frame processing for ember data extraction

## Data Flow

1. Video content → FFmpeg crop/format → `b2.py` → brightness data
2. Brightness data → `analysis.py` → statistical analysis and binary format
3. Keyboard input → headless controller → serial commands → microcontroller
4. Microcontroller → PWM signals → physical ember LEDs and heater relay