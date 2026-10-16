import { describe, expect, test } from "bun:test";
import { lowpassCoefficients, magnitudeAt } from "./biquad";

const SR = 44100;

describe("lowpass filter shapes tone the way the cutoff knob promises", () => {
  test("frequencies well above cutoff are attenuated by more than 24 dB", () => {
    const c = lowpassCoefficients(1000, 0, SR);
    const high = 20 * Math.log10(magnitudeAt(c, 8000, SR));
    expect(high).toBeLessThan(-24);
  });

  test("frequencies well below cutoff pass at unity gain", () => {
    const c = lowpassCoefficients(2000, 0, SR);
    expect(magnitudeAt(c, 100, SR)).toBeCloseTo(1, 2);
  });

  test("resonance boosts the response at the cutoff frequency", () => {
    const flat = magnitudeAt(lowpassCoefficients(1000, 0, SR), 1000, SR);
    const peaky = magnitudeAt(lowpassCoefficients(1000, 12, SR), 1000, SR);
    expect(peaky).toBeGreaterThan(flat * 3);
  });
});
