import struct

import numpy as np
import matplotlib.pyplot as plt

avg_brightness_values = np.load("avg_brightness.npy")
avg_brightness_values = avg_brightness_values - np.mean(avg_brightness_values)

# --- Analysis (FFT and Autocorrelation) ---
# (Same as in the previous version)
fft_values = np.fft.fft(avg_brightness_values)
fft_freq = np.fft.fftfreq(len(avg_brightness_values))

autocorr_values = np.correlate(
    avg_brightness_values, avg_brightness_values, mode="full"
)
autocorr_values = autocorr_values[len(autocorr_values) // 2 :]

# --- Plotting ---
# (Same as in the previous version)
plt.figure(figsize=(18, 10))

# 1. Plot of Average Brightness Over Time
plt.subplot(3, 1, 1)
plt.plot(avg_brightness_values)
plt.title("Average Brightness Over Time")
plt.xlabel("Frame Number")
plt.ylabel("Average Brightness")

# 2. Plot of FFT
plt.subplot(3, 1, 2)
plt.plot(fft_freq, np.abs(fft_values))
plt.title("FFT of Average Brightness")
plt.xlabel("Frequency")
plt.ylabel("Magnitude")
# plt.xlim(0)

# 3. Plot of Autocorrelation
plt.subplot(3, 1, 3)
plt.plot(autocorr_values)
plt.title("Autocorrelation of Average Brightness")
plt.xlabel("Lag (Frames)")
plt.ylabel("Autocorrelation")

plt.tight_layout()
plt.savefig("results/brightness_analysis.png")
plt.show()


with open("results/packed_brightness.bin", "wb") as f:
    # Write the number of values as an integer
    f.write(struct.pack("!I", len(avg_brightness_values)))
    # Write each brightness value as a float
    for value in avg_brightness_values:
        f.write(struct.pack("!f", value))
