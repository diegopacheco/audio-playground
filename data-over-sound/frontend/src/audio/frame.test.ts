import { describe, expect, test } from "bun:test";
import { byteRole, frameLength, hex, liveText, logBytes } from "./frame";
import type { ByteEvent } from "../types";

function bytes(values: number[]): ByteEvent[] {
  return values.map((value) => ({ type: "byte", value, bits: "", framing_ok: true }));
}

describe("frame byte log", () => {
  const frame = bytes([0x2d, 0xd4, 0x00, 0x02, 0x68, 0x69, 0xab, 0xcd]);

  test("length comes from the big-endian header so payload and crc are told apart", () => {
    expect(frameLength(frame)).toBe(2);
    expect(frameLength(bytes([0x2d, 0xd4, 0x01]))).toBeNull();
  });

  test("roles follow sync, length, data, crc layout", () => {
    expect(logBytes(frame).map((b) => b.role)).toEqual([
      "sync",
      "sync",
      "length",
      "length",
      "data",
      "data",
      "crc",
      "crc",
    ]);
    expect(byteRole(10, null)).toBe("data");
  });

  test("live text shows only payload bytes, never header or crc", () => {
    expect(liveText(frame)).toBe("hi");
    expect(liveText(frame.slice(0, 5))).toBe("h");
  });

  test("hex is two upper case digits", () => {
    expect(hex(0x0a)).toBe("0A");
  });
});
