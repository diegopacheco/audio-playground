import { describe, expect, test } from "bun:test";
import { PhraseRecorder } from "./recorder";

describe("phrase recorder turns key presses into notes the backend can render", () => {
  test("notes are relative to the first key press so exports start without silence", () => {
    const r = new PhraseRecorder();
    r.start(10);
    r.noteOn(60, 12);
    r.noteOn(64, 12.5);
    r.noteOff(60, 13);
    r.noteOff(64, 14);
    expect(r.finish(15)).toEqual([
      { midi: 60, start: 0, duration: 1 },
      { midi: 64, start: 0.5, duration: 1.5 },
    ]);
  });

  test("keys still held when recording stops are closed at the stop time", () => {
    const r = new PhraseRecorder();
    r.start(0);
    r.noteOn(67, 1);
    expect(r.finish(3)).toEqual([{ midi: 67, start: 0, duration: 2 }]);
  });
});
