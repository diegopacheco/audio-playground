import type { Side } from "../types";

let context: AudioContext | null = null;

function audioContext(): AudioContext {
  if (!context) context = new AudioContext();
  return context;
}

export function decodeWav(bytes: ArrayBuffer): Promise<AudioBuffer> {
  return audioContext().decodeAudioData(bytes.slice(0));
}

export class ABPlayer {
  private node: AudioBufferSourceNode | null = null;
  private startedAt = 0;
  private offset = 0;
  side: Side | null = null;

  play(buffer: AudioBuffer, side: Side, onEnd: () => void): void {
    const ctx = audioContext();
    const position = this.side ? this.offset + ctx.currentTime - this.startedAt : 0;
    this.halt();
    const node = ctx.createBufferSource();
    node.buffer = buffer;
    node.connect(ctx.destination);
    node.onended = () => {
      if (this.node === node) {
        this.halt();
        onEnd();
      }
    };
    this.offset = position < buffer.duration ? position : 0;
    this.startedAt = ctx.currentTime;
    this.node = node;
    this.side = side;
    void ctx.resume();
    node.start(0, this.offset);
  }

  stop(): void {
    this.halt();
    this.offset = 0;
  }

  private halt(): void {
    const node = this.node;
    this.node = null;
    this.side = null;
    if (node) {
      node.onended = null;
      node.stop();
      node.disconnect();
    }
  }
}
