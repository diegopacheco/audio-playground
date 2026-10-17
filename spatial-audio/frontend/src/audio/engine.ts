import type { LiveSourceKind, Position } from "../types";
import { toPanner } from "./geometry";

let context: AudioContext | null = null;

function audioContext(): AudioContext {
  if (!context) context = new AudioContext();
  return context;
}

function liveBuffer(ctx: AudioContext, kind: LiveSourceKind): AudioBuffer {
  const rate = ctx.sampleRate;
  const buffer = ctx.createBuffer(1, rate, rate);
  const data = buffer.getChannelData(0);
  const burst = Math.floor(0.006 * rate);
  const period = Math.floor(rate / 4);
  for (let n = 0; n < data.length; n++) {
    if (kind === "noise") data[n] = (Math.random() * 2 - 1) * 0.5;
    if (kind === "tone") data[n] = 0.5 * Math.sin((2 * Math.PI * 880 * n) / rate);
    if (kind === "clicks") {
      const pos = n % period;
      const env = pos < burst ? 1 - pos / burst : 0;
      data[n] = (Math.random() * 2 - 1) * env * env * 0.9;
    }
  }
  return buffer;
}

export class LivePanner {
  private node: AudioBufferSourceNode | null = null;
  private panner: PannerNode | null = null;

  get running(): boolean {
    return this.node !== null;
  }

  start(kind: LiveSourceKind, position: Position): void {
    this.stop();
    const ctx = audioContext();
    const panner = new PannerNode(ctx, {
      panningModel: "HRTF",
      distanceModel: "inverse",
      refDistance: 1,
    });
    const node = ctx.createBufferSource();
    node.buffer = liveBuffer(ctx, kind);
    node.loop = true;
    node.connect(panner).connect(ctx.destination);
    this.node = node;
    this.panner = panner;
    this.move(position);
    void ctx.resume();
    node.start();
  }

  move(position: Position): void {
    if (!this.panner) return;
    const c = toPanner(position);
    const t = audioContext().currentTime;
    this.panner.positionX.setTargetAtTime(c.x, t, 0.02);
    this.panner.positionY.setTargetAtTime(c.y, t, 0.02);
    this.panner.positionZ.setTargetAtTime(c.z, t, 0.02);
  }

  stop(): void {
    this.node?.stop();
    this.node?.disconnect();
    this.panner?.disconnect();
    this.node = null;
    this.panner = null;
  }
}

export class WavPlayer {
  private node: AudioBufferSourceNode | null = null;
  private startedAt = 0;

  async play(bytes: ArrayBuffer, onEnd: () => void): Promise<number> {
    this.stop();
    const ctx = audioContext();
    await ctx.resume();
    const buffer = await ctx.decodeAudioData(bytes.slice(0));
    const node = ctx.createBufferSource();
    node.buffer = buffer;
    node.connect(ctx.destination);
    node.onended = () => {
      if (this.node === node) {
        this.node = null;
        onEnd();
      }
    };
    this.node = node;
    this.startedAt = ctx.currentTime;
    node.start();
    return buffer.duration;
  }

  elapsed(): number {
    return audioContext().currentTime - this.startedAt;
  }

  stop(): void {
    const node = this.node;
    this.node = null;
    node?.stop();
    node?.disconnect();
  }
}
