import { BLOCK_SIZE } from "./constants";

class CaptureProcessor extends AudioWorkletProcessor {
  private block = new Float32Array(BLOCK_SIZE);
  private filled = 0;

  process(inputs: Float32Array[][]): boolean {
    const channel = inputs[0]?.[0];
    if (!channel) {
      return true;
    }
    for (const sample of channel) {
      this.block[this.filled] = sample;
      this.filled += 1;
      if (this.filled === BLOCK_SIZE) {
        this.port.postMessage(this.block, [this.block.buffer]);
        this.block = new Float32Array(BLOCK_SIZE);
        this.filled = 0;
      }
    }
    return true;
  }
}

registerProcessor("capture", CaptureProcessor);
