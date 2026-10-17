export class RingBuffer {
  private readonly data: Float32Array;
  private start = 0;
  private size = 0;

  constructor(capacity: number) {
    this.data = new Float32Array(Math.max(1, Math.floor(capacity)));
  }

  get available(): number {
    return this.size;
  }

  get capacity(): number {
    return this.data.length;
  }

  push(samples: Float32Array): void {
    for (const sample of samples) {
      if (this.size === this.data.length) {
        this.discard(1);
      }
      this.data[(this.start + this.size) % this.data.length] = sample;
      this.size += 1;
    }
  }

  pull(target: Float32Array): number {
    const count = Math.min(target.length, this.size);
    for (let i = 0; i < count; i += 1) {
      target[i] = this.data[(this.start + i) % this.data.length];
    }
    target.fill(0, count);
    this.discard(count);
    return count;
  }

  discard(count: number): void {
    const dropped = Math.min(count, this.size);
    this.start = (this.start + dropped) % this.data.length;
    this.size -= dropped;
  }
}

export class JitterBuffer {
  private readonly ring: RingBuffer;
  private playing = false;

  constructor(
    capacity: number,
    private readonly target: number,
    private readonly maximum: number,
  ) {
    this.ring = new RingBuffer(capacity);
  }

  get buffered(): number {
    return this.ring.available;
  }

  push(samples: Float32Array): void {
    this.ring.push(samples);
    if (this.ring.available > this.maximum) {
      this.ring.discard(this.ring.available - this.target);
    }
  }

  render(out: Float32Array): void {
    if (!this.playing) {
      if (this.ring.available < this.target) {
        out.fill(0);
        return;
      }
      this.playing = true;
    }
    if (this.ring.pull(out) < out.length) {
      this.playing = false;
    }
  }
}
