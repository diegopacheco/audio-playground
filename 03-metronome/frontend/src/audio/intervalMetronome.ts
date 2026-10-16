import type { BeatListener } from "../types";
import { playClick } from "./click";

export function gridErrorMs(elapsedMs: number, index: number, intervalMs: number): number {
  return elapsedMs - index * intervalMs;
}

export class IntervalMetronome {
  private timer: ReturnType<typeof setInterval> | null = null;
  private startMs = 0;
  private index = 0;

  constructor(
    private readonly ctx: AudioContext,
    private readonly bpm: number,
    private readonly beatsPerBar: number,
    private readonly onBeat: BeatListener,
  ) {}

  start(): void {
    const intervalMs = 60000 / this.bpm;
    this.startMs = performance.now();
    this.index = 0;
    this.tick(intervalMs);
    this.timer = setInterval(() => this.tick(intervalMs), intervalMs);
  }

  stop(): void {
    if (this.timer !== null) clearInterval(this.timer);
    this.timer = null;
  }

  private tick(intervalMs: number): void {
    const errorMs = gridErrorMs(performance.now() - this.startMs, this.index, intervalMs);
    playClick(this.ctx, this.ctx.currentTime, this.index % this.beatsPerBar === 0);
    this.onBeat({ index: this.index, errorMs });
    this.index += 1;
  }
}
