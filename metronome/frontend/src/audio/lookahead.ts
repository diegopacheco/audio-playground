import type { BeatListener, ScheduledBeat } from "../types";
import { playClick } from "./click";

export const LOOKAHEAD_S = 0.1;
export const TIMER_MS = 25;

export function dueBeats(
  nextIndex: number,
  startTime: number,
  interval: number,
  currentTime: number,
  lookahead: number,
): ScheduledBeat[] {
  const beats: ScheduledBeat[] = [];
  let index = nextIndex;
  while (startTime + index * interval < currentTime + lookahead) {
    beats.push({ index, time: startTime + index * interval });
    index += 1;
  }
  return beats;
}

export function lateMs(scheduledTime: number, currentTime: number): number {
  return Math.max(0, currentTime - scheduledTime) * 1000;
}

export class LookaheadMetronome {
  private timer: ReturnType<typeof setInterval> | null = null;
  private startTime = 0;
  private nextIndex = 0;

  constructor(
    private readonly ctx: AudioContext,
    private readonly bpm: number,
    private readonly beatsPerBar: number,
    private readonly onBeat: BeatListener,
  ) {}

  start(): void {
    this.startTime = this.ctx.currentTime + LOOKAHEAD_S;
    this.nextIndex = 0;
    this.schedule();
    this.timer = setInterval(() => this.schedule(), TIMER_MS);
  }

  stop(): void {
    if (this.timer !== null) clearInterval(this.timer);
    this.timer = null;
  }

  private schedule(): void {
    const now = this.ctx.currentTime;
    const beats = dueBeats(this.nextIndex, this.startTime, 60 / this.bpm, now, LOOKAHEAD_S);
    for (const beat of beats) {
      playClick(this.ctx, beat.time, beat.index % this.beatsPerBar === 0);
      const errorMs = lateMs(beat.time, now);
      const delayMs = Math.max(0, (beat.time - now) * 1000);
      setTimeout(() => this.onBeat({ index: beat.index, errorMs }), delayMs);
      this.nextIndex = beat.index + 1;
    }
  }
}
