import { describe, expect, test } from "bun:test";
import { parsePcm16Wav } from "./wav";
import { spectrogram } from "./spectrogram";

function pcm16Wav(samples: number[], sampleRate: number): ArrayBuffer {
  const buffer = new ArrayBuffer(44 + samples.length * 2);
  const view = new DataView(buffer);
  const write = (offset: number, text: string) =>
    [...text].forEach((c, i) => view.setUint8(offset + i, c.charCodeAt(0)));
  write(0, "RIFF");
  view.setUint32(4, 36 + samples.length * 2, true);
  write(8, "WAVE");
  write(12, "fmt ");
  view.setUint32(16, 16, true);
  view.setUint16(20, 1, true);
  view.setUint16(22, 1, true);
  view.setUint32(24, sampleRate, true);
  view.setUint32(28, sampleRate * 2, true);
  view.setUint16(32, 2, true);
  view.setUint16(34, 16, true);
  write(36, "data");
  view.setUint32(40, samples.length * 2, true);
  samples.forEach((s, i) => view.setInt16(44 + i * 2, Math.round(s * 32767), true));
  return buffer;
}

describe("wav parsing for the backend modulate response", () => {
  test("reads sample rate and scaled samples", () => {
    const audio = parsePcm16Wav(pcm16Wav([0, 0.5, -0.5], 48000));
    expect(audio.sampleRate).toBe(48000);
    expect(audio.samples.length).toBe(3);
    expect(audio.samples[1]).toBeCloseTo(0.5, 3);
    expect(audio.samples[2]).toBeCloseTo(-0.5, 3);
  });

  test("rejects non wav data", () => {
    expect(() => parsePcm16Wav(new ArrayBuffer(20))).toThrow();
  });
});

describe("spectrogram display", () => {
  test("a 1200 Hz mark tone lights up the 1200 Hz row brightest", () => {
    const sampleRate = 48000;
    const samples = Float32Array.from({ length: 4800 }, (_, n) =>
      Math.sin((2 * Math.PI * 1200 * n) / sampleRate),
    );
    const spec = spectrogram({ sampleRate, samples }, 4000, 80);
    const frame = spec.values.subarray(0, spec.bins);
    const brightest = frame.indexOf(Math.max(...frame));
    const hz = ((brightest + 0.5) * spec.maxHz) / spec.bins;
    expect(Math.abs(hz - 1200)).toBeLessThan(60);
  });

  test("max frequency is capped below Nyquist for low sample rates", () => {
    const spec = spectrogram({ sampleRate: 6000, samples: new Float32Array(2000) }, 4000, 10);
    expect(spec.maxHz).toBeLessThan(3000);
  });
});
