import { describe, expect, test } from "bun:test";
import { formatBytes, formatDuration, toDbfs } from "./format";

describe("toDbfs", () => {
  test("full scale is 0 dBFS so clipping-level files read as the ceiling", () => {
    expect(toDbfs(1)).toBe("0.00 dBFS");
  });

  test("halving amplitude drops about 6 dB, matching the backend CLI", () => {
    expect(toDbfs(0.5)).toBe("-6.02 dBFS");
  });

  test("digital silence is minus infinity instead of NaN", () => {
    expect(toDbfs(0)).toBe("-inf dBFS");
  });
});

describe("formatDuration", () => {
  test("keeps millisecond precision so short clips are not shown as zero", () => {
    expect(formatDuration(0.25)).toBe("0:00.250");
  });

  test("rolls seconds into minutes", () => {
    expect(formatDuration(75.5)).toBe("1:15.500");
  });
});

describe("formatBytes", () => {
  test("chooses a unit that keeps the number readable", () => {
    expect(formatBytes(512)).toBe("512 B");
    expect(formatBytes(2048)).toBe("2.0 KB");
    expect(formatBytes(3 * 1024 * 1024)).toBe("3.00 MB");
  });
});
