import type { ByteEvent, ByteRole, LoggedByte } from "../types";

const SYNC_BYTES = 2;
const HEADER_BYTES = 4;

export function frameLength(bytes: ByteEvent[]): number | null {
  if (bytes.length < HEADER_BYTES) return null;
  return (bytes[2].value << 8) | bytes[3].value;
}

export function byteRole(index: number, length: number | null): ByteRole {
  if (index < SYNC_BYTES) return "sync";
  if (index < HEADER_BYTES) return "length";
  if (length === null || index < HEADER_BYTES + length) return "data";
  return "crc";
}

export function logBytes(bytes: ByteEvent[]): LoggedByte[] {
  const length = frameLength(bytes);
  return bytes.map((b, index) => ({ ...b, index, role: byteRole(index, length) }));
}

export function liveText(bytes: ByteEvent[]): string {
  const values = logBytes(bytes)
    .filter((b) => b.role === "data")
    .map((b) => b.value);
  return new TextDecoder().decode(new Uint8Array(values));
}

export function hex(value: number): string {
  return value.toString(16).toUpperCase().padStart(2, "0");
}
