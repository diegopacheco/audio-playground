export class Framer {
  private buffer: Float32Array;
  private filled = 0;

  constructor(readonly size: number) {
    this.buffer = new Float32Array(size);
  }

  push(chunk: Float32Array): Float32Array[] {
    const frames: Float32Array[] = [];
    let offset = 0;
    while (offset < chunk.length) {
      const take = Math.min(this.size - this.filled, chunk.length - offset);
      this.buffer.set(chunk.subarray(offset, offset + take), this.filled);
      this.filled += take;
      offset += take;
      if (this.filled === this.size) {
        frames.push(this.buffer);
        this.buffer = new Float32Array(this.size);
        this.filled = 0;
      }
    }
    return frames;
  }
}
