import { describe, expect, test } from "bun:test";
import { errorMessage, formatTimestamp, pickRecorderMime } from "./format";

describe("formatTimestamp", () => {
  test("whisper offsets keep millisecond precision so segments line up with audio", () => {
    expect(formatTimestamp(0)).toBe("00:00.000");
    expect(formatTimestamp(2.5)).toBe("00:02.500");
    expect(formatTimestamp(5.12)).toBe("00:05.120");
  });

  test("minutes roll over past 59 seconds instead of showing 60+ seconds", () => {
    expect(formatTimestamp(61.25)).toBe("01:01.250");
    expect(formatTimestamp(600)).toBe("10:00.000");
  });

  test("negative values clamp to zero", () => {
    expect(formatTimestamp(-3)).toBe("00:00.000");
  });
});

describe("errorMessage", () => {
  test("backend error json is surfaced so a missing key 503 is readable in the UI", () => {
    const msg = errorMessage(503, { error: "ANTHROPIC_API_KEY is not set on the backend" });
    expect(msg).toBe("503: ANTHROPIC_API_KEY is not set on the backend");
  });

  test("non json bodies still produce a status message", () => {
    expect(errorMessage(500, null)).toBe("500: request failed");
    expect(errorMessage(502, { message: "x" })).toBe("502: request failed");
  });
});

describe("pickRecorderMime", () => {
  test("prefers opus webm which the backend decodes with ffmpeg", () => {
    expect(pickRecorderMime(() => true)).toBe("audio/webm;codecs=opus");
  });

  test("falls back to the first supported type, or browser default", () => {
    expect(pickRecorderMime((m) => m === "audio/mp4")).toBe("audio/mp4");
    expect(pickRecorderMime(() => false)).toBe("");
  });
});
