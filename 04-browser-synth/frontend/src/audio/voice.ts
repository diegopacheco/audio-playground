import type { Preset } from "../types";
import { scheduleAttack, scheduleRelease } from "./envelope";
import { midiToFrequency } from "./notes";

export class Voice {
  private readonly osc: OscillatorNode;
  private readonly filter: BiquadFilterNode;
  private readonly amp: GainNode;
  private readonly startTime: number;

  constructor(
    private readonly ctx: AudioContext,
    destination: AudioNode,
    midi: number,
    private readonly preset: Preset,
  ) {
    this.osc = new OscillatorNode(ctx, { type: preset.waveform, frequency: midiToFrequency(midi) });
    this.filter = new BiquadFilterNode(ctx, { type: "lowpass", frequency: preset.cutoff, Q: preset.resonance });
    this.amp = new GainNode(ctx, { gain: 0 });
    this.osc.connect(this.filter).connect(this.amp).connect(destination);
    this.startTime = ctx.currentTime;
    scheduleAttack(this.amp.gain, preset, this.startTime, preset.gain);
    this.osc.start(this.startTime);
  }

  release(): void {
    const end = scheduleRelease(this.amp.gain, this.preset, this.startTime, this.ctx.currentTime, this.preset.gain);
    this.osc.stop(end + 0.02);
    this.osc.onended = () => this.amp.disconnect();
  }
}
