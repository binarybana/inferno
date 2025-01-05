import sys
import numpy as np

# Region dimensions
width = 50
height = 50
frame_size = width * height  # For grayscale

# Process frame by frame
avg_brightness_values = []
frame_num = 0

# Read from stdin in a loop, frame by frame
with sys.stdin.buffer as stdin_buffer:
    while True:
        # Read one frame's worth of data
        frame_data = stdin_buffer.read(frame_size)

        # Check for end of input
        if not frame_data:
            break

        # Ensure we read a full frame
        if len(frame_data) != frame_size:
            print(f"Warning: Incomplete frame {frame_num} read, skipping.")
            continue

        frame_num += 1
        # Skip the fade-in
        if frame_num < 200:
            continue

        # Process the frame
        frame = np.frombuffer(frame_data, dtype=np.uint8).reshape((height, width))
        avg_brightness = np.mean(frame)
        avg_brightness_values.append(avg_brightness)

        if frame_num > 10000:
            break

np.save("results/avg_brightness.npy", avg_brightness_values)
