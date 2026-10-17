import type { Rect, RollLayout, Song } from "../types";

const PADDING = 2;

export function rollLayout(song: Song, width: number, height: number): RollLayout {
  const pitches = [...song.notes.map((n) => n.pitch), ...song.chords.flatMap((c) => c.pitches)];
  const low = pitches.length ? Math.min(...pitches) - PADDING : 48;
  const high = pitches.length ? Math.max(...pitches) + PADDING : 84;
  return { width, height, low, high, beats: song.bars * 4 };
}

export function pitchRect(layout: RollLayout, pitch: number, start: number, duration: number): Rect {
  const rows = layout.high - layout.low + 1;
  const rowHeight = layout.height / rows;
  const beatWidth = layout.width / layout.beats;
  return {
    x: start * beatWidth,
    y: (layout.high - pitch) * rowHeight,
    width: duration * beatWidth,
    height: rowHeight,
  };
}

export function isBlackKey(pitch: number): boolean {
  return [1, 3, 6, 8, 10].includes(((pitch % 12) + 12) % 12);
}

const NAMES = ["C", "C#", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B"];

export function pitchName(pitch: number): string {
  return `${NAMES[((pitch % 12) + 12) % 12]}${Math.floor(pitch / 12) - 1}`;
}
