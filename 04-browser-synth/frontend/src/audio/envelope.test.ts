import { describe, expect, test } from "bun:test";
import { levelAt } from "./envelope";
import type { Adsr } from "../types";

const adsr: Adsr = { attack: 0.1, decay: 0.2, sustain: 0.5, release: 0.4 };

describe("ADSR levels match the Rust renderer so exports sound like live play", () => {
  test("attack ramps linearly from silence to full level", () => {
    expect(levelAt(adsr, 0)).toBeCloseTo(0);
    expect(levelAt(adsr, 0.05)).toBeCloseTo(0.5);
    expect(levelAt(adsr, 0.1)).toBeCloseTo(1);
  });

  test("decay falls from peak to sustain level", () => {
    expect(levelAt(adsr, 0.2)).toBeCloseTo(0.75);
    expect(levelAt(adsr, 0.3)).toBeCloseTo(0.5);
  });

  test("sustain holds for as long as the key is down", () => {
    expect(levelAt(adsr, 5)).toBeCloseTo(0.5);
  });

  test("release fades to silence after the key is lifted", () => {
    expect(levelAt(adsr, 1.2, 1)).toBeCloseTo(0.25);
    expect(levelAt(adsr, 1.4, 1)).toBeCloseTo(0);
    expect(levelAt(adsr, 2, 1)).toBe(0);
  });

  test("lifting during attack releases from the current level without a click", () => {
    expect(levelAt(adsr, 0.05, 0.05)).toBeCloseTo(0.5);
    expect(levelAt(adsr, 0.25, 0.05)).toBeCloseTo(0.25);
  });

  test("zero attack jumps straight to peak instead of dividing by zero", () => {
    expect(levelAt({ ...adsr, attack: 0 }, 0)).toBeCloseTo(1);
  });
});
