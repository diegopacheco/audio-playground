import { describe, expect, test } from "bun:test";
import { RoundTripTracker, SILENCE_DB, meterPercent, rms, samplesToMs, toDb } from "./metrics";
import { voiceSocketUrl } from "../api";

describe("levels", () => {
  test("a full-scale sine reads -3 dBFS, the standard reference for meters", () => {
    const sine = new Float32Array(4800).map((_, i) => Math.sin((2 * Math.PI * 440 * i) / 48000));
    expect(toDb(rms(sine))).toBeCloseTo(-3.01, 1);
  });

  test("silence floors at the meter bottom instead of -Infinity", () => {
    expect(toDb(rms(new Float32Array(128)))).toBe(SILENCE_DB);
    expect(meterPercent(SILENCE_DB)).toBe(0);
    expect(meterPercent(0)).toBe(100);
    expect(meterPercent(-30)).toBe(50);
  });
});

describe("latency", () => {
  test("pairs replies with blocks in send order because the server answers one to one", () => {
    const tracker = new RoundTripTracker(1);
    tracker.sent(0);
    tracker.sent(10);
    expect(tracker.received(25)).toBe(25);
    expect(tracker.received(40)).toBe(30);
  });

  test("smooths jitter so the display does not flicker", () => {
    const tracker = new RoundTripTracker(0.5);
    tracker.sent(0);
    tracker.received(20);
    tracker.sent(100);
    tracker.received(140);
    expect(tracker.average).toBe(30);
  });

  test("converts the backend STFT latency from samples to milliseconds", () => {
    expect(samplesToMs(2304, 48000)).toBeCloseTo(48, 5);
  });
});

describe("voiceSocketUrl", () => {
  test("upgrades the page scheme to ws and carries effect, semitones and rate", () => {
    const url = new URL(voiceSocketUrl("http://localhost:5113", { effect: "deep", semitones: -6 }, 44100.4));
    expect(url.protocol).toBe("ws:");
    expect(url.pathname).toBe("/api/voice");
    expect(url.searchParams.get("effect")).toBe("deep");
    expect(url.searchParams.get("semitones")).toBe("-6");
    expect(url.searchParams.get("rate")).toBe("44100");
    expect(voiceSocketUrl("https://host", { effect: "pitch", semitones: 1 }, 48000).startsWith("wss:")).toBe(true);
  });
});
