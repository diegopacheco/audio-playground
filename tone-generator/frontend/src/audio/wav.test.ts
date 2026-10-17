import { describe, expect, test } from "bun:test";
import { HEADER_LEN, headerHex, parseWavHeader } from "./wavHeader";
import { pcmSamples, toPoints, visibleSampleCount } from "./samples";

function wav(sampleRate: number, samples: number[]): ArrayBuffer {
  const buf = new ArrayBuffer(HEADER_LEN + samples.length * 2);
  const v = new DataView(buf);
  const text = (o: number, s: string) => [...s].forEach((c, i) => v.setUint8(o + i, c.charCodeAt(0)));
  text(0, "RIFF");
  v.setUint32(4, 36 + samples.length * 2, true);
  text(8, "WAVE");
  text(12, "fmt ");
  v.setUint32(16, 16, true);
  v.setUint16(20, 1, true);
  v.setUint16(22, 1, true);
  v.setUint32(24, sampleRate, true);
  v.setUint32(28, sampleRate * 2, true);
  v.setUint16(32, 2, true);
  v.setUint16(34, 16, true);
  text(36, "data");
  v.setUint32(40, samples.length * 2, true);
  samples.forEach((s, i) => v.setInt16(HEADER_LEN + i * 2, s, true));
  return buf;
}

describe("parseWavHeader", () => {
  test("reads little-endian fields so the panel shows what the backend really encoded", () => {
    const info = parseWavHeader(wav(44100, [0, 1, 2]));
    expect(info.sampleRate).toBe(44100);
    expect(info.channels).toBe(1);
    expect(info.bitsPerSample).toBe(16);
    expect(info.dataSize).toBe(6);
    expect(info.fields).toHaveLength(13);
    expect(info.fields.find((f) => f.name === "ByteRate")?.value).toBe("88200");
  });

  test("fields tile the 44 bytes with no gap so every byte is explained", () => {
    const { fields } = parseWavHeader(wav(8000, []));
    let next = 0;
    for (const f of fields) {
      expect(f.offset).toBe(next);
      next += f.size;
    }
    expect(next).toBe(HEADER_LEN);
  });

  test("rejects non-WAV data instead of showing garbage fields", () => {
    expect(() => parseWavHeader(new ArrayBuffer(10))).toThrow();
    expect(() => parseWavHeader(new ArrayBuffer(HEADER_LEN))).toThrow("not a RIFF/WAVE file");
  });

  test("hex dump starts with the RIFF magic bytes", () => {
    expect(headerHex(wav(8000, [])).slice(0, 4)).toEqual(["52", "49", "46", "46"]);
  });
});

describe("samples", () => {
  test("decodes signed 16-bit PCM so negative half-waves draw below the axis", () => {
    const buf = wav(8000, [32767, -32768, -1, 0]);
    expect(Array.from(pcmSamples(buf, 8))).toEqual([32767, -32768, -1, 0]);
  });

  test("visible window shows a fixed number of periods regardless of pitch", () => {
    expect(visibleSampleCount(44100, 441, 4, 100000)).toBe(400);
    expect(visibleSampleCount(44100, 882, 4, 100000)).toBe(200);
    expect(visibleSampleCount(44100, 20, 4, 1000)).toBe(1000);
  });

  test("points map full scale to the canvas edges and zero to the middle", () => {
    const pts = toPoints(new Int16Array([0, 32767, -32768]), 3, 200, 100);
    expect(pts[0]).toEqual([0, 50]);
    expect(pts[1][1]).toBeCloseTo(0, 1);
    expect(pts[2]).toEqual([200, 100]);
  });
});
