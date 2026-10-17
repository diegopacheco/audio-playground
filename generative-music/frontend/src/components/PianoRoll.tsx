import { useEffect, useRef } from "react";
import type { Song } from "../types";
import { isBlackKey, pitchName, pitchRect, rollLayout } from "../audio/roll";

interface Props {
  song: Song;
  playhead: number | null;
}

const WIDTH = 940;
const HEIGHT = 380;
const GUTTER = 36;

function draw(canvas: HTMLCanvasElement, song: Song, playhead: number | null): void {
  const ctx = canvas.getContext("2d");
  if (!ctx) return;
  const ratio = window.devicePixelRatio || 1;
  canvas.width = WIDTH * ratio;
  canvas.height = HEIGHT * ratio;
  ctx.setTransform(ratio, 0, 0, ratio, 0, 0);
  ctx.clearRect(0, 0, WIDTH, HEIGHT);
  const layout = rollLayout(song, WIDTH - GUTTER, HEIGHT);

  ctx.save();
  ctx.translate(GUTTER, 0);
  for (let pitch = layout.low; pitch <= layout.high; pitch++) {
    const row = pitchRect(layout, pitch, 0, layout.beats);
    ctx.fillStyle = isBlackKey(pitch) ? "#f1f3f7" : "#ffffff";
    ctx.fillRect(row.x, row.y, row.width, row.height);
  }
  for (let beat = 0; beat <= layout.beats; beat++) {
    const x = (beat / layout.beats) * layout.width;
    ctx.strokeStyle = beat % 4 === 0 ? "#c4cad6" : "#e6e9ef";
    ctx.beginPath();
    ctx.moveTo(x, 0);
    ctx.lineTo(x, HEIGHT);
    ctx.stroke();
  }
  song.chords.forEach((chord, index) => {
    ctx.fillStyle = index % 2 === 0 ? "rgba(110, 150, 225, 0.45)" : "rgba(155, 120, 225, 0.45)";
    chord.pitches.forEach((pitch) => {
      const r = pitchRect(layout, pitch, chord.start, chord.duration);
      ctx.fillRect(r.x + 1, r.y + 1, r.width - 2, r.height - 2);
    });
    const label = pitchRect(layout, layout.low, chord.start, chord.duration);
    ctx.fillStyle = "#59627a";
    ctx.font = "12px system-ui, sans-serif";
    ctx.fillText(chord.symbol, label.x + 4, HEIGHT - 6);
  });
  song.notes.forEach((note) => {
    const r = pitchRect(layout, note.pitch, note.start, note.duration);
    ctx.fillStyle = `rgba(232, 110, 80, ${0.45 + note.velocity / 254})`;
    ctx.fillRect(r.x + 1, r.y + 1, Math.max(2, r.width - 2), Math.max(2, r.height - 2));
  });
  if (playhead !== null) {
    const x = (playhead / layout.beats) * layout.width;
    ctx.strokeStyle = "#2b7a4b";
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(x, 0);
    ctx.lineTo(x, HEIGHT);
    ctx.stroke();
  }
  ctx.restore();

  ctx.fillStyle = "#7a8399";
  ctx.font = "10px system-ui, sans-serif";
  for (let pitch = layout.low; pitch <= layout.high; pitch++) {
    if (pitch % 12 !== 0) continue;
    const r = pitchRect(layout, pitch, 0, 1);
    ctx.fillText(pitchName(pitch), 2, r.y + r.height - 1);
  }
}

export function PianoRoll({ song, playhead }: Props) {
  const ref = useRef<HTMLCanvasElement>(null);
  useEffect(() => {
    if (ref.current) draw(ref.current, song, playhead);
  }, [song, playhead]);
  return <canvas ref={ref} className="roll" style={{ width: WIDTH, height: HEIGHT }} />;
}
