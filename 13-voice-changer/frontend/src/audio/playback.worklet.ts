import { BLOCK_SIZE, MAX_BUFFER_SECONDS, PREBUFFER_BLOCKS, RING_SECONDS, STATUS_EVERY_QUANTA } from "./constants";
import { JitterBuffer } from "./ringBuffer";

class PlaybackProcessor extends AudioWorkletProcessor {
  private readonly buffer = new JitterBuffer(
    sampleRate * RING_SECONDS,
    BLOCK_SIZE * PREBUFFER_BLOCKS,
    sampleRate * MAX_BUFFER_SECONDS,
  );
  private quanta = 0;

  constructor() {
    super();
    this.port.onmessage = (event: MessageEvent<Float32Array>) => this.buffer.push(event.data);
  }

  process(_inputs: Float32Array[][], outputs: Float32Array[][]): boolean {
    const channels = outputs[0] ?? [];
    const first = channels[0];
    if (first) {
      this.buffer.render(first);
      for (const channel of channels.slice(1)) {
        channel.set(first);
      }
    }
    this.quanta += 1;
    if (this.quanta % STATUS_EVERY_QUANTA === 0) {
      this.port.postMessage(this.buffer.buffered);
    }
    return true;
  }
}

registerProcessor("playback", PlaybackProcessor);
