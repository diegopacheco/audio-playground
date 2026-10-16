import { INSTRUMENTS } from "../pattern";
import type { Cursor, PlaybackSettings } from "../types";
import { VOICES } from "./instruments";
import { stepsInWindow } from "./timing";

const LOOKAHEAD_SECONDS = 0.1;
const TICK_MS = 25;

export class Sequencer {
  private ctx: AudioContext | null = null;
  private master: GainNode | null = null;
  private timer: number | null = null;
  private highlights: number[] = [];
  private cursor: Cursor = { nextStep: 0, nextBaseTime: 0 };

  constructor(
    private readonly settings: () => PlaybackSettings,
    private readonly onStep: (step: number) => void,
  ) {}

  async start(): Promise<void> {
    if (this.timer !== null) return;
    this.ctx ??= new AudioContext();
    await this.ctx.resume();
    this.master = this.ctx.createGain();
    this.master.gain.value = 0.6;
    this.master.connect(this.ctx.destination);
    this.cursor = { nextStep: 0, nextBaseTime: this.ctx.currentTime + 0.05 };
    this.timer = window.setInterval(() => this.tick(), TICK_MS);
    this.tick();
  }

  stop(): void {
    if (this.timer !== null) window.clearInterval(this.timer);
    this.timer = null;
    this.highlights.forEach((h) => window.clearTimeout(h));
    this.highlights = [];
    this.master?.disconnect();
    this.master = null;
    this.onStep(-1);
  }

  private tick(): void {
    const ctx = this.ctx;
    const master = this.master;
    if (!ctx || !master) return;
    const { pattern, bpm, swing } = this.settings();
    const batch = stepsInWindow(this.cursor, ctx.currentTime + LOOKAHEAD_SECONDS, bpm, swing);
    this.cursor = batch.cursor;
    for (const { step, time } of batch.steps) {
      for (const name of INSTRUMENTS) {
        if (pattern.tracks[name][step]) VOICES[name](ctx, master, time);
      }
      const delay = Math.max(0, (time - ctx.currentTime) * 1000);
      const handle = window.setTimeout(() => this.onStep(step), delay);
      this.highlights.push(handle);
      if (this.highlights.length > 64) this.highlights.shift();
    }
  }
}
