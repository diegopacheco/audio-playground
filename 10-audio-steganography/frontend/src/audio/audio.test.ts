import { describe, expect, test } from "bun:test";
import { parseWav } from "./wav";
import { columns, diffSamples, diffStats, utf8Length } from "./diff";

function wav(samples: number[], channels = 1, sampleRate = 44100): ArrayBuffer {
  const buffer = new ArrayBuffer(44 + samples.length * 2);
  const view = new DataView(buffer);
  const text = (o: number, s: string) => [...s].forEach((c, i) => view.setUint8(o + i, c.charCodeAt(0)));
  text(0, "RIFF");
  view.setUint32(4, 36 + samples.length * 2, true);
  text(8, "WAVEfmt ");
  view.setUint32(16, 16, true);
  view.setUint16(20, 1, true);
  view.setUint16(22, channels, true);
  view.setUint32(24, sampleRate, true);
  view.setUint32(28, sampleRate * channels * 2, true);
  view.setUint16(32, channels * 2, true);
  view.setUint16(34, 16, true);
  text(36, "data");
  view.setUint32(40, samples.length * 2, true);
  samples.forEach((s, i) => view.setInt16(44 + i * 2, s, true));
  return buffer;
}

describe("parseWav", () => {
  test("reads signed samples exactly so the diff shows single LSB changes", () => {
    const audio = parseWav(wav([0, 1, -1, 32767, -32768]));
    expect(audio.sampleRate).toBe(44100);
    expect([...audio.samples]).toEqual([0, 1, -1, 32767, -32768]);
  });

  test("downmixes stereo like the backend so both sides compare the same signal", () => {
    expect([...parseWav(wav([100, 300, -50, 50], 2)).samples]).toEqual([200, 0]);
  });

  test("rejects non wav bytes", () => {
    expect(() => parseWav(new TextEncoder().encode("hello world!").buffer)).toThrow();
  });
});

describe("diff", () => {
  test("an lsb change of one shows as max diff 1 on exactly the touched samples", () => {
    const original = Int16Array.from([1000, -1000, 1000, -1000]);
    const stego = Int16Array.from([1001, -1000, 1001, -1000]);
    const stats = diffStats(original, stego);
    expect(stats.maxDiff).toBe(1);
    expect(stats.changed).toBe(2);
    expect([...diffSamples(original, stego)]).toEqual([1, 0, 1, 0]);
  });

  test("identical audio reports zero changes so the UI can say nothing was hidden", () => {
    const s = Int16Array.from([5, 6, 7]);
    expect(diffStats(s, s)).toEqual({ maxDiff: 0, changed: 0 });
  });

  test("columns keep peaks when squeezing many samples into few pixels", () => {
    const cols = columns(Int16Array.from([0, 9, 0, 0, -7, 0, 0, 0]), 2);
    expect(cols).toEqual([
      { min: 0, max: 9 },
      { min: -7, max: 0 },
    ]);
  });

  test("capacity counts utf-8 bytes, not characters", () => {
    expect(utf8Length("abc")).toBe(3);
    expect(utf8Length("é")).toBe(2);
  });
});
