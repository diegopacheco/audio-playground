import type { PcmAudio } from "../types";

function tag(view: DataView, offset: number): string {
  return String.fromCharCode(
    view.getUint8(offset),
    view.getUint8(offset + 1),
    view.getUint8(offset + 2),
    view.getUint8(offset + 3),
  );
}

export function parsePcm16Wav(buffer: ArrayBuffer): PcmAudio {
  const view = new DataView(buffer);
  if (buffer.byteLength < 12 || tag(view, 0) !== "RIFF" || tag(view, 8) !== "WAVE") {
    throw new Error("not a RIFF/WAVE file");
  }
  let offset = 12;
  let sampleRate = 0;
  let channels = 1;
  while (offset + 8 <= buffer.byteLength) {
    const id = tag(view, offset);
    const size = view.getUint32(offset + 4, true);
    const start = offset + 8;
    if (id === "fmt ") {
      if (view.getUint16(start, true) !== 1 || view.getUint16(start + 14, true) !== 16) {
        throw new Error("only 16-bit PCM wav is supported");
      }
      channels = view.getUint16(start + 2, true);
      sampleRate = view.getUint32(start + 4, true);
    } else if (id === "data") {
      if (sampleRate === 0) throw new Error("data chunk before fmt chunk");
      const frames = Math.floor(Math.min(size, buffer.byteLength - start) / (2 * channels));
      const samples = new Float32Array(frames);
      for (let i = 0; i < frames; i++) {
        let sum = 0;
        for (let c = 0; c < channels; c++) {
          sum += view.getInt16(start + (i * channels + c) * 2, true) / 32768;
        }
        samples[i] = sum / channels;
      }
      return { sampleRate, samples };
    }
    offset = start + size + (size % 2);
  }
  throw new Error("no data chunk");
}
