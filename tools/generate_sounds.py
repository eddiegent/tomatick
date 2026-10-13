#!/usr/bin/env python3
"""Generates the built-in alert sounds into public/sounds/.

The sounds are synthesized here, so they have no third-party licence: they
are covered by the project's MIT licence like the rest of the repository.
Only the standard library is used. Run it from the repository root:

    python3 tools/generate_sounds.py

The output is deterministic, so running it again gives identical files.
"""

import math
import struct
import sys
import wave
from pathlib import Path

RATE = 22050  # Hz. Plenty for soft chimes, and keeps the files small.
PEAK = 0.8  # Of full scale, so the sounds never clip.


def silence(seconds):
    return [0.0] * int(RATE * seconds)


def tone(freq, seconds, partials, decay, attack=0.004):
    """One note: a sum of decaying sine partials.

    `partials` is a list of (frequency ratio, amplitude, decay speed factor).
    `decay` is the exponential decay rate in 1/s of the loudest partial.
    """
    count = int(RATE * seconds)
    samples = [0.0] * count
    for ratio, amplitude, speed in partials:
        step = 2 * math.pi * freq * ratio / RATE
        rate = decay * speed
        for n in range(count):
            t = n / RATE
            samples[n] += amplitude * math.sin(step * n) * math.exp(-rate * t)
    # A short fade in and out avoids clicks.
    fade_in = max(1, int(RATE * attack))
    fade_out = max(1, int(RATE * 0.02))
    for n in range(min(fade_in, count)):
        samples[n] *= n / fade_in
    for n in range(min(fade_out, count)):
        samples[count - 1 - n] *= n / fade_out
    return samples


def mix(*tracks):
    """Overlays tracks that start at offsets: (offset in seconds, samples)."""
    length = max(int(RATE * offset) + len(samples) for offset, samples in tracks)
    out = [0.0] * length
    for offset, samples in tracks:
        start = int(RATE * offset)
        for n, value in enumerate(samples):
            out[start + n] += value
    return out


def chime():
    """Two soft rising notes, like a door chime."""
    soft = [(1.0, 1.0, 1.0), (2.0, 0.25, 1.6), (3.0, 0.08, 2.4)]
    return mix(
        (0.0, tone(784.0, 1.1, soft, decay=3.2)),  # G5
        (0.28, tone(1046.5, 1.4, soft, decay=2.8)),  # C6
    )


def bell():
    """A single bell with the inharmonic partials of struck metal."""
    metal = [
        (1.0, 1.0, 1.0),
        (2.0, 0.6, 1.3),
        (2.76, 0.45, 1.8),
        (5.4, 0.25, 2.6),
        (8.93, 0.12, 3.6),
    ]
    return tone(523.25, 2.2, metal, decay=1.9, attack=0.002)  # C5


def beep():
    """Two short, gentle beeps."""
    pure = [(1.0, 1.0, 1.0), (2.0, 0.08, 1.0)]

    def one():
        note = tone(660.0, 0.16, pure, decay=2.0, attack=0.01)
        return note

    return mix((0.0, one()), (0.26, one()))


def marimba():
    """A quick rising arpeggio of wooden notes."""
    wood = [(1.0, 1.0, 1.0), (4.0, 0.35, 4.5), (9.2, 0.08, 8.0)]
    notes = [523.25, 659.25, 783.99, 1046.5]  # C5 E5 G5 C6
    return mix(
        *[
            (index * 0.15, tone(freq, 0.7, wood, decay=5.5, attack=0.002))
            for index, freq in enumerate(notes)
        ]
    )


def normalize(samples):
    top = max(abs(value) for value in samples) or 1.0
    return [value / top * PEAK for value in samples]


def write_wav(path, samples):
    with wave.open(str(path), "wb") as wav:
        wav.setnchannels(1)
        wav.setsampwidth(2)
        wav.setframerate(RATE)
        wav.writeframes(
            b"".join(
                struct.pack("<h", int(round(max(-1.0, min(1.0, value)) * 32767)))
                for value in samples
            )
        )


SOUNDS = {
    "chime": chime,
    "bell": bell,
    "beep": beep,
    "marimba": marimba,
}


def main():
    root = Path(__file__).resolve().parent.parent
    out = root / "public" / "sounds"
    out.mkdir(parents=True, exist_ok=True)
    for name, make in SOUNDS.items():
        path = out / f"{name}.wav"
        write_wav(path, normalize(make()))
        print(f"{path.relative_to(root)}  {path.stat().st_size / 1024:.0f} KB")
    return 0


if __name__ == "__main__":
    sys.exit(main())
