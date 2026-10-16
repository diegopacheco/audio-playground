import type { PcmAudio } from "../types";

function tag(view: DataView, offset: number): string {
  return String.fromCharCode(
    view.getUint8(offset),
    view.getUint8(offset + 1),
    view.getUint8(offset + 2),
    view.getUint8(offset + 3),
  );
}

export function parseWav(buffer: ArrayBuffer): PcmAudio {
  const view = new DataView(buffer);
  if (buffer.byteLength < 12 || tag(view, 0) !== "RIFF" || tag(view, 8) !== "WAVE") {
    throw new Error("not a RIFF/WAVE file");
  }
  let pos = 12;
  let channels = 0;
  let sampleRate = 0;
  while (pos + 8 <= buffer.byteLength) {
    const id = tag(view, pos);
    const size = view.getUint32(pos + 4, true);
    const start = pos + 8;
    if (id === "fmt ") {
      if (view.getUint16(start, true) !== 1 || view.getUint16(start + 14, true) !== 16) {
        throw new Error("only 16-bit PCM is supported");
      }
      channels = view.getUint16(start + 2, true);
      sampleRate = view.getUint32(start + 4, true);
    } else if (id === "data") {
      if (channels === 0) throw new Error("data chunk before fmt chunk");
      const end = Math.min(start + size, buffer.byteLength);
      const frames = Math.floor((end - start) / (2 * channels));
      const samples = new Int16Array(frames);
      for (let f = 0; f < frames; f++) {
        let sum = 0;
        for (let c = 0; c < channels; c++) {
          sum += view.getInt16(start + (f * channels + c) * 2, true);
        }
        samples[f] = Math.trunc(sum / channels);
      }
      return { sampleRate, samples };
    }
    pos = start + size + (size % 2);
  }
  throw new Error("no data chunk");
}
