import type { PcmAudio, Spectrogram } from "../types";

export function binPower(frame: Float32Array, freq: number, sampleRate: number): number {
  const coeff = 2 * Math.cos((2 * Math.PI * freq) / sampleRate);
  let s1 = 0;
  let s2 = 0;
  for (let n = 0; n < frame.length; n++) {
    const w = 0.5 - 0.5 * Math.cos((2 * Math.PI * n) / (frame.length - 1));
    const s0 = frame[n] * w + coeff * s1 - s2;
    s2 = s1;
    s1 = s0;
  }
  return Math.max(0, s1 * s1 + s2 * s2 - coeff * s1 * s2);
}

export function spectrogram(
  audio: PcmAudio,
  maxHz = 4000,
  bins = 80,
  frameSize = 512,
  maxFrames = 400,
): Spectrogram {
  const cappedHz = Math.min(maxHz, audio.sampleRate / 2 - 1);
  const total = Math.max(0, audio.samples.length - frameSize);
  const hop = Math.max(1, Math.ceil(total / maxFrames));
  const frames = Math.floor(total / hop) + 1;
  const values = new Float32Array(frames * bins);
  let peak = 1e-12;
  for (let f = 0; f < frames; f++) {
    const frame = audio.samples.subarray(f * hop, f * hop + frameSize);
    for (let b = 0; b < bins; b++) {
      const power = binPower(frame, ((b + 0.5) * cappedHz) / bins, audio.sampleRate);
      values[f * bins + b] = power;
      peak = Math.max(peak, power);
    }
  }
  for (let i = 0; i < values.length; i++) {
    values[i] = Math.max(0, 1 + Math.log10(values[i] / peak + 1e-12) / 6);
  }
  return { frames, bins, maxHz: cappedHz, values };
}
