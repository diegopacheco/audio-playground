import type { Note } from "../types";

export class PhraseRecorder {
  private origin = 0;
  private readonly open = new Map<number, number>();
  private readonly done: Note[] = [];

  start(now: number): void {
    this.origin = now;
    this.open.clear();
    this.done.length = 0;
  }

  noteOn(midi: number, now: number): void {
    if (!this.open.has(midi)) this.open.set(midi, now - this.origin);
  }

  noteOff(midi: number, now: number): void {
    const start = this.open.get(midi);
    if (start === undefined) return;
    this.open.delete(midi);
    this.done.push({ midi, start, duration: Math.max(now - this.origin - start, 0.01) });
  }

  finish(now: number): Note[] {
    for (const midi of [...this.open.keys()]) this.noteOff(midi, now);
    const first = Math.min(...this.done.map((n) => n.start));
    return this.done
      .map((n) => ({ ...n, start: n.start - first }))
      .sort((a, b) => a.start - b.start);
  }
}
