import { expect, test } from "bun:test";
import { formatHz } from "./format";

test("bin widths of large FFTs keep a decimal so 8192 and 4096 at 48 kHz stay distinguishable", () => {
  expect(formatHz(48000 / 8192)).toBe("5.9 Hz");
  expect(formatHz(48000 / 4096)).toBe("11.7 Hz");
});

test("audible tones switch to kHz above 1000 Hz", () => {
  expect(formatHz(440)).toBe("440 Hz");
  expect(formatHz(1585)).toBe("1.58 kHz");
});
