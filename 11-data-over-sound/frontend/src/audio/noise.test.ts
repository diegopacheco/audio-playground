import { describe, expect, test } from "bun:test";
import { addWhiteNoise, seededRandom, signalPower, withLeadingSilence } from "./noise";

function tone(len: number): Float32Array {
  return Float32Array.from({ length: len }, (_, n) => 0.6 * Math.sin(n * 0.3));
}

describe("loopback channel noise", () => {
  test("added noise power matches the requested SNR so the slider means what it says", () => {
    const clean = tone(48000);
    const noisy = addWhiteNoise(clean, 10, seededRandom(1));
    const noise = noisy.map((v, i) => v - clean[i]);
    const measured = 10 * Math.log10(signalPower(clean) / signalPower(noise));
    expect(Math.abs(measured - 10)).toBeLessThan(0.3);
  });

  test("lower SNR adds more noise", () => {
    const clean = tone(8000);
    const power = (snr: number) =>
      signalPower(addWhiteNoise(clean, snr, seededRandom(3)).map((v, i) => v - clean[i]));
    expect(power(0)).toBeGreaterThan(power(20) * 50);
  });

  test("leading silence shifts the signal without changing it", () => {
    const clean = tone(10);
    const padded = withLeadingSilence(clean, 5);
    expect(padded.length).toBe(15);
    expect(Array.from(padded.subarray(0, 5))).toEqual([0, 0, 0, 0, 0]);
    expect(Array.from(padded.subarray(5))).toEqual(Array.from(clean));
  });

  test("seeded random stays inside (0, 1) so log() in Box-Muller never sees zero", () => {
    const random = seededRandom(42);
    for (let i = 0; i < 10000; i++) {
      const v = random();
      expect(v).toBeGreaterThan(0);
      expect(v).toBeLessThan(1);
    }
  });
});
