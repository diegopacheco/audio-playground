import type { HeaderField, WavInfo } from "../types";

export const HEADER_LEN = 44;

function ascii(view: DataView, offset: number): string {
  let out = "";
  for (let i = 0; i < 4; i++) out += String.fromCharCode(view.getUint8(offset + i));
  return out;
}

export function parseWavHeader(buffer: ArrayBuffer): WavInfo {
  if (buffer.byteLength < HEADER_LEN) throw new Error("file is shorter than a 44-byte WAV header");
  const v = new DataView(buffer);
  if (ascii(v, 0) !== "RIFF" || ascii(v, 8) !== "WAVE") throw new Error("not a RIFF/WAVE file");
  const u32 = (o: number) => v.getUint32(o, true);
  const u16 = (o: number) => v.getUint16(o, true);
  const fields: HeaderField[] = [
    { offset: 0, size: 4, name: "ChunkID", value: ascii(v, 0), meaning: "Marks the file as RIFF container" },
    { offset: 4, size: 4, name: "ChunkSize", value: String(u32(4)), meaning: "File size minus 8 bytes" },
    { offset: 8, size: 4, name: "Format", value: ascii(v, 8), meaning: "RIFF form type is WAVE" },
    { offset: 12, size: 4, name: "Subchunk1ID", value: ascii(v, 12), meaning: "Start of the format chunk" },
    { offset: 16, size: 4, name: "Subchunk1Size", value: String(u32(16)), meaning: "16 bytes of format data for PCM" },
    { offset: 20, size: 2, name: "AudioFormat", value: String(u16(20)), meaning: "1 means uncompressed PCM" },
    { offset: 22, size: 2, name: "NumChannels", value: String(u16(22)), meaning: "1 is mono, 2 is stereo" },
    { offset: 24, size: 4, name: "SampleRate", value: String(u32(24)), meaning: "Samples per second" },
    { offset: 28, size: 4, name: "ByteRate", value: String(u32(28)), meaning: "SampleRate x NumChannels x BitsPerSample / 8" },
    { offset: 32, size: 2, name: "BlockAlign", value: String(u16(32)), meaning: "Bytes per sample frame across all channels" },
    { offset: 34, size: 2, name: "BitsPerSample", value: String(u16(34)), meaning: "Bit depth of each sample" },
    { offset: 36, size: 4, name: "Subchunk2ID", value: ascii(v, 36), meaning: "Start of the sample data chunk" },
    { offset: 40, size: 4, name: "Subchunk2Size", value: String(u32(40)), meaning: "Number of bytes of sample data" },
  ];
  return {
    sampleRate: u32(24),
    channels: u16(22),
    bitsPerSample: u16(34),
    dataSize: u32(40),
    fields,
  };
}

export function headerHex(buffer: ArrayBuffer): string[] {
  return Array.from(new Uint8Array(buffer, 0, HEADER_LEN), (b) => b.toString(16).padStart(2, "0"));
}
