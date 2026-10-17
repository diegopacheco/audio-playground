import type { Preset } from "../types";
import { Voice } from "./voice";

export const MASTER_LEVEL = 0.3;

export class Synth {
  private ctx: AudioContext | null = null;
  private master: GainNode | null = null;
  private readonly voices = new Map<number, Voice>();

  constructor(private preset: Preset) {}

  setPreset(preset: Preset): void {
    this.preset = preset;
  }

  noteOn(midi: number): void {
    if (this.voices.has(midi)) return;
    const { ctx, master } = this.ensureContext();
    this.voices.set(midi, new Voice(ctx, master, midi, this.preset));
  }

  noteOff(midi: number): void {
    const voice = this.voices.get(midi);
    if (!voice) return;
    voice.release();
    this.voices.delete(midi);
  }

  allNotesOff(): void {
    for (const midi of [...this.voices.keys()]) this.noteOff(midi);
  }

  private ensureContext(): { ctx: AudioContext; master: GainNode } {
    if (!this.ctx || !this.master) {
      this.ctx = new AudioContext();
      this.master = new GainNode(this.ctx, { gain: MASTER_LEVEL });
      this.master.connect(this.ctx.destination);
    }
    if (this.ctx.state === "suspended") void this.ctx.resume();
    return { ctx: this.ctx, master: this.master };
  }
}
